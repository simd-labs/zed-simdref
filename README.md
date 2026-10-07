# zed-simdref

![simdref inlay hints in a .s file in Zed](https://raw.githubusercontent.com/simd-labs/zed-simdref/screenshots/zed-asm.png)
![simdref inlay hints in a .cpp file in Zed](https://raw.githubusercontent.com/simd-labs/zed-simdref/screenshots/zed-cpp.png)

A Zed extension that starts `simdref-lsp` for Assembly, C, and C++ buffers. The server sends SIMD instruction text as inlay hints at the right of each line. Zed gets inlay hints from all language servers on a buffer and merges them, so simdref runs adjacent to clangd. Hover on an instruction shows the full simdref page from the local catalog, with no network. Zed shows it in the hover popup.

To get hints in `.s` files, install the Zed `assembly` extension. That extension adds the Assembly language to Zed. Without it, Zed does not parse `.s` files and simdref does not start in them. Hints in C and C++ buffers work without it.

## Install

The extension is not on the Zed extension registry. Use the dev install.

Zed must have this setting in `settings.json`:

```json
"inlay_hints": {"enabled": true}
```

simdref 0.0.8 or newer (on PyPI) has inlay hints. The extension installs simdref.

## Auto-install

The extension looks for `simdref-lsp` on the `PATH` first. If the extension finds none, it downloads the `uv` release for the current platform from `astral-sh/uv`. It checks the download against the `.sha256` file from the same GitHub release. This detects a corrupt download, but not a tampered release. It then runs `uv tool install simdref` and `isa update`. All files stay in the extension work directory of Zed. The extension does not change the `PATH`. The first start runs `isa update`, which makes the catalog.

## Manual install

Run these commands, then start Zed again:

```sh
uv tool install simdref
isa update
```

`pip install simdref` also works. Run `isa update` after it. Instructions: https://github.com/simd-labs/simdref

If the auto-install stops with an error, the language server status menu in Zed shows this text, followed by the cause:

```text
simdref not found. Install it: uv tool install simdref (or pip install simdref), then run isa update. Instructions: https://github.com/simd-labs/simdref
```

The extension writes each command it runs (uv download, `uv tool install simdref`, `isa update`) to `install.log` in the extension work directory. The log holds the arguments, environment, exit status, stdout, and stderr. Open that file first when the install stops with an error.

## Development

In Zed, run `zed: install dev extension` and select this directory.

## Troubleshooting

If simdref-lsp is on PATH, the extension uses it. simdref 0.0.8 or newer has inlay hints (`uv tool upgrade simdref`).

## License

GPL-3.0-or-later, the same license as simdref. See `LICENSE`.
