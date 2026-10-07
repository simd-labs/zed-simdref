# Podman test rig

The container runs Zed and headless Sway in rootless podman. No host display is necessary.

## Build the image

Make the image one time. It must have network access and about 1.3 GB of disk.

```sh
podman build -t zed-shot test/podman
```

## Build the extension

```sh
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/zed_simdref.wasm extension.wasm
```

## Install check

The container has no `simdref` on the `PATH`, so the extension installs it. The default wait is 600 seconds.

```sh
sh test/podman/run-install.sh [FILE [OUT.png [WAIT_SECS]]]
```

`FILE` defaults to `test/podman/ws/demo.s`. Use `test/podman/ws/demo.cpp` for C++. `OUT.png` defaults to `test/podman/out/install.png`.

The script also copies `OUT.Zed.log` and `OUT.install.simdref.log`, the extension `install.log`, adjacent to `OUT.png`.

The script prints this line on success and exits with 0:

```text
starting language server process. binary path: .../extensions/work/simdref/bin/simdref-lsp
```

## Screenshot check

`shot.sh` opens a file in Zed and writes a screenshot of the inlay hints.

```sh
sh test/podman/shot.sh WORKSPACE FILE OUT.png [DEV_EXT_DIR ...]
```

`WORKSPACE` and `FILE` are host paths. `FILE` is below `WORKSPACE`. Each `DEV_EXT_DIR` holds `extension.toml` and `extension.wasm`. Its name must be the extension id, `simdref`.

Environment settings:

- `WAIT_SECS`: seconds to wait before the screenshot, default 12.
- `SETTINGS_JSON`: Zed settings file, default `test/podman/settings.json`.
- `PODMAN_NETWORK`: podman network mode, default `slirp4netns`.

For hints, the image must have `simdref-lsp` 0.0.8 or newer on the `PATH`. The base image does not.

## Update the screenshots

The `screenshots` branch holds the top-level README screenshots. Replace its single commit and force-push. Do not commit PNG files to `main`.

## Exit status

The container kills Zed and Sway, so an exit status of 137 or a kill message is usual. Read the logs, not the podman exit status.
