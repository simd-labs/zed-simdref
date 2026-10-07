# Podman test rig

The rig runs Zed in a rootless podman container. Sway runs headless inside the container, so no host display is used. The scripts never pass `DISPLAY` or `WAYLAND_DISPLAY`.

## Build the image

Build the image once. It needs network access and about 1.3 GB of disk.

```sh
podman build -t zed-shot test/podman
```

## Build the extension

Both checks load `extension.wasm` from the repository root.

```sh
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/zed_simdref.wasm extension.wasm
```

## Install check

The container has no `simdref` on the `PATH`. The extension downloads uv, runs `uv tool install simdref` and then runs `isa update`. This takes minutes. The default wait is 600 seconds.

```sh
sh test/podman/run-install.sh [FILE [OUT.png [WAIT_SECS]]]
```

`FILE` defaults to `test/podman/ws/demo.s`. Use `test/podman/ws/demo.cpp` for C++. `OUT.png` defaults to `test/podman/out/install.png`. Git ignores `test/podman/out/`.

The script copies the logs next to `OUT.png`. `OUT.Zed.log` is the Zed log and `OUT.install.simdref.log` is the `install.log` of the extension.

Success is this line in `OUT.Zed.log`, and the script prints it and exits with 0:

```text
starting language server process. binary path: .../extensions/work/simdref/bin/simdref-lsp
```

## Screenshot check

`shot.sh` opens a file in Zed and writes a screenshot. Use it to look at the inlay hints.

```sh
sh test/podman/shot.sh WORKSPACE FILE OUT.png [DEV_EXT_DIR ...]
```

`WORKSPACE` and `FILE` are host paths, and `FILE` is under `WORKSPACE`. The script resolves relative paths. Each `DEV_EXT_DIR` is a directory with `extension.toml` and `extension.wasm`. The directory name must be the extension id, `simdref`.

Environment variables:

- `WAIT_SECS`: seconds to wait before the screenshot. The default is 12.
- `SETTINGS_JSON`: Zed settings file. The default is `test/podman/settings.json`.
- `ZED_SHOT_IMAGE`: image name. The default is `zed-shot:latest`.
- `PODMAN_NETWORK`: podman network mode. The default is `slirp4netns`.

For hints, `simdref-lsp` must be on the `PATH` in the image, in version 0.0.8 or newer. The base image does not have it.

## Update the screenshots

To update the screenshots in the main README, replace the single commit on the `screenshots` branch. It is an orphan branch. Force-push it. Never commit PNGs to `main`.

## Exit status

The rig kills Zed and sway at the end of the run. An exit status of 137 or a kill message at the end is normal. Read the logs, not the exit status of podman.
