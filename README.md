# zed-simdref

![simdref inlay hints in a .s file in Zed](https://raw.githubusercontent.com/simd-labs/zed-simdref/screenshots/zed-asm.png)
![simdref inlay hints in a .cpp file in Zed](https://raw.githubusercontent.com/simd-labs/zed-simdref/screenshots/zed-cpp.png)

A Zed extension that starts `simdref-lsp` for Assembly, C, and C++ buffers. The server shows SIMD instruction text as inlay hints at the right of each line. Hover on an instruction shows the full simdref page from the local catalog, with no network.

To get hints in `.s` files, install the Zed `assembly` extension.

## Install

The extension is not on the Zed extension registry. Use the dev install.

Zed must have this setting in `settings.json`:

```json
"inlay_hints": {"enabled": true}
```

simdref 0.0.8 or newer has inlay hints. The extension installs simdref.

## Auto-install

The extension looks for `simdref-lsp` on the `PATH` first. If it finds none, it downloads the `uv` release from `astral-sh/uv` and checks it against the `.sha256` file from the same release. It then runs `uv tool install simdref` and `isa update`. All files stay in the extension work directory of Zed.

## Manual install

Run these commands, then start Zed again:

```sh
uv tool install simdref
isa update
```

`pip install simdref` also works. Run `isa update` after it. Instructions: https://github.com/simd-labs/simdref

If the install stops with an error, the language server status menu in Zed shows the cause. The file `install.log` in the extension work directory holds the arguments, environment, exit status, stdout, and stderr of each command the extension runs. Open it first.

## Development

In Zed, run `zed: install dev extension` and select this directory.

## Troubleshooting

If simdref-lsp is on PATH, the extension uses it. simdref 0.0.8 or newer has inlay hints (`uv tool upgrade simdref`).

## License

GPL-3.0-or-later, the same license as simdref. See `LICENSE`.
