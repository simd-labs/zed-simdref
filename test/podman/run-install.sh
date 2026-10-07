#!/bin/sh
# run-install.sh [FILE [OUT.png [WAIT_SECS]]]
# Install check. Run Zed with this extension as a dev extension, no simdref
# on PATH, network on. The extension downloads uv, runs "uv tool install
# simdref", then "isa update". Needs extension.wasm in the repo root:
#   cargo build --release --target wasm32-wasip2
#   cp target/wasm32-wasip2/release/zed_simdref.wasm extension.wasm
# Exit 0 if Zed.log shows the language server starting from the work dir.
set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
FILE=$(realpath "${1:-$HERE/ws/demo.s}")
OUT=${2:-$HERE/out/install.png}
export WAIT_SECS=${3:-600}

[ -f "$ROOT/extension.wasm" ] || { echo "missing $ROOT/extension.wasm, build it first" >&2; exit 2; }

# Zed reads the extension as <id>, so stage the two files under that name.
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
mkdir "$STAGE/simdref"
cp "$ROOT/extension.toml" "$ROOT/extension.wasm" "$STAGE/simdref/"

sh "$HERE/shot.sh" "$(dirname "$FILE")" "$FILE" "$OUT" "$STAGE/simdref" || true

grep -E 'starting language server process.*extensions/work/simdref/bin/simdref-lsp' "${OUT%.png}.Zed.log"
