use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use zed_extension_api::{
    self as zed, process::Command, set_language_server_installation_status as set_status,
    Architecture, DownloadedFileType, LanguageServerId, LanguageServerInstallationStatus as Status,
    Os, Result,
};

const ERR_HINT: &str = "simdref not found. Install it: uv tool install simdref (or pip install simdref), then run isa update. Instructions: https://github.com/simd-labs/simdref";

/// Pinned uv release. Bump UV_TAG to move to a newer uv.
const UV_TAG: &str = "0.12.23";

struct Simdref;

/// True when a regular file or a symlink exists at `path`. The WASI sandbox
/// refuses to follow uv's absolute tool shim symlinks outside their parent
/// preopen (`metadata` returns EPERM), so use `symlink_metadata`. Zed runs the
/// binary natively where the link resolves fine.
fn is_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_file() || m.file_type().is_symlink())
}

/// True when `stamp` is missing or older than 24 h.
fn needs_check(stamp: &Path) -> bool {
    const DAY: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);
    fs::metadata(stamp)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_none_or(|elapsed| elapsed > DAY)
}

/// True when `before` and `after` are both Some and differ.
fn is_newer(before: Option<&str>, after: Option<&str>) -> bool {
    matches!((before, after), (Some(b), Some(a)) if b != a)
}

/// Returns `isa --version` stdout, or None on any failure.
fn version(isa: &Path, env: &[(&str, &str)]) -> Option<String> {
    let out = Command::new(isa.to_string_lossy())
        .args(["--version"])
        .envs(env.iter().copied())
        .output()
        .ok()?;
    (out.status == Some(0))
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Release asset stem and archive suffix of uv for a platform.
fn uv_asset(os: Os, arch: Architecture) -> (String, &'static str) {
    let arch = match arch {
        Architecture::Aarch64 => "aarch64",
        Architecture::X86 => "i686",
        Architecture::X8664 => "x86_64",
    };
    let (os, suffix) = match os {
        Os::Mac => ("apple-darwin", "tar.gz"),
        Os::Linux => ("unknown-linux-gnu", "tar.gz"),
        Os::Windows => ("pc-windows-msvc", "zip"),
    };
    (format!("uv-{arch}-{os}"), suffix)
}

fn run(program: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<()> {
    let out = Command::new(program.to_string_lossy())
        .args(args.iter().copied())
        .envs(env.iter().copied())
        .output()?;
    // Keep a full record of each command in install.log inside the work dir.
    // It is the first place a user looks when an install fails.
    let log = env::current_dir()
        .map_err(|e| e.to_string())?
        .join("install.log");
    let record = format!(
        "cmd: {} {}\nenv: {:?}\nstatus: {:?}\nstdout: {}\nstderr: {}\n---\n",
        program.display(),
        args.join(" "),
        env,
        out.status,
        String::from_utf8_lossy(&out.stdout).trim(),
        String::from_utf8_lossy(&out.stderr).trim()
    );
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
        .and_then(|mut f| std::io::Write::write_all(&mut f, record.as_bytes()))
        .ok();
    if out.status == Some(0) {
        return Ok(());
    }
    Err(format!(
        "{} {} exited with {:?}: {}",
        program.display(),
        args.join(" "),
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

/// Returns the path of the uv binary of the extracted archive in `work/uv`.
/// The tar.gz holds a top-level `uv-TRIPLE/` directory. The zip is flat.
fn uv_path(os: Os, stem: &str, work: &Path) -> PathBuf {
    let dir = work.join("uv");
    match os {
        Os::Windows => dir.join("uv.exe"),
        _ => dir.join(stem).join("uv"),
    }
}

/// Returns the uv binary in the work dir. It downloads uv when it is missing.
fn ensure_uv(id: &LanguageServerId, work: &Path) -> Result<PathBuf> {
    let (os, arch) = zed::current_platform();
    let (stem, suffix) = uv_asset(os, arch);
    let uv = uv_path(os, &stem, work);
    if is_file(&uv) {
        return Ok(uv);
    }
    set_status(id, &Status::CheckingForUpdate);
    let name = format!("{stem}.{suffix}");
    let url =
        format!("https://github.com/astral-sh/uv/releases/download/{UV_TAG}/{name}");
    set_status(id, &Status::Downloading);
    // Record the pinned download in install.log. zed::download_file is a host
    // call, so `run` never sees it.
    let record = format!("download: {url}\nsha256: {url}.sha256\n---\n");
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(work.join("install.log"))
        .and_then(|mut f| std::io::Write::write_all(&mut f, record.as_bytes()))
        .ok();
    let result = download_extract(&url, &name, work)
        .map_err(|cause| format!("failed to download {name}: {cause}"));
    if result.is_err() {
        fs::remove_file(work.join(&name)).ok();
        fs::remove_file(work.join(format!("{name}.sha256"))).ok();
        fs::remove_dir_all(work.join("uv")).ok();
    }
    result?;
    if is_file(&uv) {
        Ok(uv)
    } else {
        Err(format!("{} missing after download", uv.display()))
    }
}

/// Downloads `url`, checks the SHA-256 of the bytes against `<url>.sha256`,
/// then extracts the verified archive with the system `tar` into `work/uv`.
fn download_extract(url: &str, name: &str, work: &Path) -> Result<()> {
    zed::download_file(
        &format!("{url}.sha256"),
        &format!("{name}.sha256"),
        DownloadedFileType::Uncompressed,
    )?;
    let digest = fs::read_to_string(work.join(format!("{name}.sha256")))
        .map_err(|e| format!("cannot read {name}.sha256: {e}"))?;
    let want = digest.split_whitespace().next().unwrap_or("").to_string();
    zed::download_file(url, name, DownloadedFileType::Uncompressed)?;
    let bytes = fs::read(work.join(name)).map_err(|e| format!("cannot read {name}: {e}"))?;
    compare(&hex(&Sha256::digest(&bytes)), &want)?;
    fs::create_dir_all(work.join("uv")).map_err(|e| e.to_string())?;
    // Windows 10+ ships bsdtar, which also reads .zip archives. Zed runs
    // process:exec outside the work dir, so both paths are absolute and no
    // -P is needed. Dropping -P keeps tar from honoring absolute member paths.
    // The capability args must match this call exactly: ["-xf", ARCHIVE, "-C", DIR].
    let dir = work.join("uv");
    let archive = work.join(name);
    run(
        Path::new("tar"),
        &["-xf", &*archive.to_string_lossy(), "-C", &*dir.to_string_lossy()],
        &[],
    )?;
    fs::remove_file(work.join(name)).ok();
    Ok(())
}

/// Errors with 'checksum mismatch' when the two lowercase hex digests differ.
fn compare(got: &str, want: &str) -> Result<()> {
    if got.len() == 64 && got == want {
        Ok(())
    } else {
        Err("checksum mismatch".to_string())
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Returns the environment for every command the extension runs. It keeps
/// every uv write inside the work dir.
fn work_env(work: &Path) -> Vec<(String, String)> {
    [
        ("UV_TOOL_DIR", work.join("tools")),
        ("UV_TOOL_BIN_DIR", work.join("bin")),
        ("UV_PYTHON_INSTALL_DIR", work.join("python")),
        ("UV_CACHE_DIR", work.join("uv-cache")),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string_lossy().into_owned()))
    .collect()
}

/// Returns the simdref-lsp path. It installs simdref into the work dir when needed.
fn install(id: &LanguageServerId) -> Result<PathBuf> {
    let work = env::current_dir().map_err(|e| e.to_string())?;
    let windows = matches!(zed::current_platform().0, Os::Windows);
    let exe = |name: &str| if windows { format!("{name}.exe") } else { name.to_string() };
    let (tools, bin) = (work.join("tools"), work.join("bin"));
    let lsp = bin.join(exe("simdref-lsp"));
    if is_file(&lsp) {
        return Ok(lsp);
    }
    let uv = ensure_uv(id, &work)?;
    set_status(id, &Status::Downloading);
    let env_pairs = work_env(&work);
    let env: Vec<(&str, &str)> = env_pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let result = run(&uv, &["tool", "install", "simdref"], &env)
        .and_then(|()| run(&bin.join(exe("isa")), &["update"], &env))
        .and_then(|()| {
            if is_file(&lsp) {
                Ok(())
            } else {
                Err(format!("{} missing after install", lsp.display()))
            }
        });
    if result.is_err() {
        // A half install must not pass the is_file check on the next start.
        fs::remove_dir_all(&tools).ok();
        fs::remove_dir_all(&bin).ok();
    }
    result.map(|()| lsp)
}

/// Upgrades the simdref copy the extension installed in its work dir. A stamp
/// file throttles the check to once a day. It calls `isa vaddps --short` to
/// refresh the catalog only when the version changed. Every error is ignored:
/// an older simdref is better than none.
fn upgrade() -> Option<()> {
    let work = env::current_dir().map_err(|e| e.to_string()).ok()?;
    let (os, arch) = zed::current_platform();
    let exe = |name: &str| if matches!(os, Os::Windows) { format!("{name}.exe") } else { name.to_string() };
    let bin = work.join("bin");
    let isa = bin.join(exe("isa"));
    let uv = uv_path(os, &uv_asset(os, arch).0, &work);
    if !is_file(&isa) || !is_file(&uv) {
        return None;
    }
    // shortcut: the stamp uses file mtime, not wall time, so a fs clock jump
    // can delay the check by a day. Upgrade to SystemTime when it matters.
    let stamp = work.join("last-update-check");
    if !needs_check(&stamp) {
        return None;
    }
    fs::File::create(&stamp).ok()?;
    let env_pairs = work_env(&work);
    let env: Vec<(&str, &str)> = env_pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let before = version(&isa, &env);
    run(&uv, &["tool", "upgrade", "simdref"], &env).ok()?;
    let after = version(&isa, &env);
    if is_newer(before.as_deref(), after.as_deref()) {
        run(&isa, &["vaddps", "--short"], &env).ok()?;
    }
    Some(())
}

impl zed::Extension for Simdref {
    fn new() -> Self {
        Simdref
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Only the copy the extension installed gets upgraded; a PATH install
        // stays as it is. The check runs before the server start command
        // returns, so it delays the start; the stamp file throttles the delay
        // to one network check a day.
        if worktree.which("simdref-lsp").is_none() {
            upgrade();
        }
        let command = match worktree.which("simdref-lsp") {
            Some(path) => path,
            None => match install(id) {
                Ok(path) => {
                    set_status(id, &Status::None);
                    path.to_string_lossy().into_owned()
                }
                Err(e) => {
                    let msg = format!("{ERR_HINT} Cause: {e}");
                    set_status(id, &Status::Failed(msg.clone()));
                    return Err(msg);
                }
            },
        };
        Ok(zed::Command {
            command,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(Simdref);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_file_accepts_a_symlink_shim() {
        // uv tool shim is a symlink on Unix. The extension runs under WASI,
        // which refuses to follow absolute links (EPERM). is_file must still
        // find the shim so the install check does not delete a good install.
        let dir = env::temp_dir().join(format!("simdref-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target");
        fs::write(&target, b"#!/bin/sh\n").unwrap();
        let link = dir.join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &link).unwrap();
        assert!(is_file(&target));
        assert!(is_file(&link));
        assert!(!is_file(&dir.join("missing")));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn uv_asset_names_match_the_release() {
        let (stem, suffix) = uv_asset(Os::Linux, Architecture::X8664);
        assert_eq!(format!("{stem}.{suffix}"), "uv-x86_64-unknown-linux-gnu.tar.gz");
        let (stem, suffix) = uv_asset(Os::Mac, Architecture::Aarch64);
        assert_eq!(format!("{stem}.{suffix}"), "uv-aarch64-apple-darwin.tar.gz");
        let (stem, suffix) = uv_asset(Os::Windows, Architecture::X8664);
        assert_eq!(format!("{stem}.{suffix}"), "uv-x86_64-pc-windows-msvc.zip");
    }

    #[test]
    fn uv_path_matches_the_archive_layout() {
        let work = Path::new("/work");
        // The tar.gz nests uv-TRIPLE/uv.
        let (stem, _) = uv_asset(Os::Linux, Architecture::X8664);
        assert_eq!(
            uv_path(Os::Linux, &stem, work),
            PathBuf::from("/work/uv/uv-x86_64-unknown-linux-gnu/uv")
        );
        let (stem, _) = uv_asset(Os::Mac, Architecture::Aarch64);
        assert_eq!(
            uv_path(Os::Mac, &stem, work),
            PathBuf::from("/work/uv/uv-aarch64-apple-darwin/uv")
        );
        // The Windows zip is flat: uv.exe at the top level.
        let (stem, _) = uv_asset(Os::Windows, Architecture::X8664);
        assert_eq!(
            uv_path(Os::Windows, &stem, work),
            PathBuf::from("/work/uv/uv.exe")
        );
    }

    #[test]
    fn compare_accepts_a_matching_hash() {
        let got = hex(&Sha256::digest(b"uv archive"));
        assert!(compare(&got, &got).is_ok());
    }

    #[test]
    fn compare_rejects_a_bad_hash() {
        let got = hex(&Sha256::digest(b"uv archive"));
        let other = hex(&Sha256::digest(b"other"));
        let err = compare(&other, &got).unwrap_err();
        assert_eq!(err, "checksum mismatch");
        assert!(compare(&got, &got[..32]).is_err());
    }

    #[test]
    fn needs_check_when_stamp_is_old_or_missing() {
        let dir = env::temp_dir().join(format!("simdref-stamp-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let stamp = dir.join("last-update-check");
        // A missing stamp means a check.
        assert!(needs_check(&stamp));
        // An old stamp means a check.
        fs::write(&stamp, b"").unwrap();
        let old = filetime::FileTime::from_unix_time(946_684_800, 0); // 2000-01-01
        filetime::set_file_mtime(&stamp, old).unwrap();
        assert!(needs_check(&stamp));
        // A fresh stamp skips the check.
        filetime::set_file_mtime(&stamp, filetime::FileTime::now()).unwrap();
        assert!(!needs_check(&stamp));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn is_newer_only_on_a_version_change() {
        assert!(is_newer(Some("0.1.0"), Some("0.2.0")));
        assert!(!is_newer(Some("0.2.0"), Some("0.2.0")));
        assert!(!is_newer(None, Some("0.2.0")));
        assert!(!is_newer(Some("0.1.0"), None));
    }
}
