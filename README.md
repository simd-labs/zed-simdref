# zed-simdref

![simdref inlay hints in a .s file in Zed](https://raw.githubusercontent.com/simd-labs/zed-simdref/screenshots/zed-asm.png)
![simdref inlay hints in a .cpp file in Zed](https://raw.githubusercontent.com/simd-labs/zed-simdref/screenshots/zed-cpp.png)

A Zed extension that starts `simdref-lsp` for Assembly, C and C++ buffers. The server returns SIMD instruction reference text as inlay hints at the right of each line. Zed asks every language server on a buffer for inlay hints and merges them, so simdref runs next to clangd.

The extension needs the Zed `assembly` extension to get hints in `.s` files. That extension defines the Assembly language. Without it, Zed does not parse `.s` files and simdref does not start in them. Hints in C and C++ buffers do not need it.

## Install

The extension is not published yet. Use the dev install below.

Zed needs this setting in `settings.json`:

```json
"inlay_hints": {"enabled": true}
```

Inlay hints need simdref 0.0.8 or newer (on PyPI). The extension installs it for you.

### Auto-install

The extension looks for `simdref-lsp` on the `PATH` first. If it finds none, it downloads the `uv` release for the current platform from `astral-sh/uv`. It checks the download against the `.sha256` file from the same GitHub release. This detects a corrupt download, but not a tampered release. It then runs `uv tool install simdref` and `isa update`. All files go into the extension work directory of Zed. The extension does not change the `PATH`. The first start runs `isa update`, which builds the catalog.

### Manual install

Run these commands, then restart Zed:

```sh
uv tool install simdref
isa update
```

`pip install simdref` also works. Run `isa update` after it. Instructions: https://github.com/simd-labs/simdref

If the auto-install fails, the language server status menu in Zed shows this text, followed by the cause:

```text
simdref not found. Install it: uv tool install simdref (or pip install simdref), then run isa update. Instructions: https://github.com/simd-labs/simdref
```

Each command the extension runs (uv download extract, `uv tool install simdref`, `isa update`) is logged with its arguments, environment, exit status, stdout and stderr to `install.log` inside the extension work directory. Open that file first when the install fails.

## Development

In Zed, run `zed: install dev extension` and select this directory.

## Troubleshooting

If simdref-lsp is on PATH, the extension uses it; inlay hints need simdref 0.0.8 or newer (`uv tool upgrade simdref`).

## License

GPL-3.0-or-later, the same license as simdref. See `LICENSE`.
