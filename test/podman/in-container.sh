#!/bin/sh
# in-container.sh FILE WAIT SETTINGS
set -eu

FILE="$1"
WAIT="$2"
SETTINGS="$3"

export XDG_RUNTIME_DIR=/tmp/xdg-$$
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"

export WLR_BACKENDS=headless
export WLR_RENDERER=pixman
export WLR_LIBINPUT_NO_DEVICES=1
export LIBGL_ALWAYS_SOFTWARE=1
export ZED_ALLOW_EMULATED_GPU=1
# mesa vulkan software
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json

UD=/home/shot/zed-user-data
mkdir -p "$UD/config"
cp "$SETTINGS" "$UD/config/settings.json"

# prestage dev extensions into user data dir
for d in /dev-ext-in/*; do
  [ -d "$d" ] || continue
  name=$(basename "$d")
  target=$(readlink -f "$d")
  mkdir -p "$UD/extensions/installed"
  ln -s "$target" "$UD/extensions/installed/$name"
done

sway --unsupported-gpu >/shot/sway.log 2>&1 &
SWAY_PID=$!

wait_sway_socket() {
  i=0
  while [ $i -lt 50 ]; do
    if [ -S "$XDG_RUNTIME_DIR/wayland-1" ]; then return 0; fi
    i=$((i+1)); sleep 0.1
  done
  return 1
}
WAYLAND_DISPLAY=wayland-1
export WAYLAND_DISPLAY
wait_sway_socket || { echo "sway socket never appeared"; cat /shot/sway.log; exit 3; }

mkdir -p "$(dirname "$FILE")"
/opt/zed.app/libexec/zed-editor --user-data-dir "$UD" "$FILE" >/shot/zed.log 2>&1 &
ZED_PID=$!

sleep "$WAIT"

grim /shot/shot.png
rc=$?
find "$UD/extensions" -maxdepth 3 >/shot/extensions.txt 2>&1 || true
find "$UD/extensions/work" -maxdepth 5 >/shot/workdir.txt 2>&1 || true
{ ls -la /home/shot/.cache/uv /home/shot/.local/share/uv; } >/shot/home-uv.txt 2>&1 || true
cp "$UD/extensions/index.json" /shot/index.json 2>/dev/null || true
cp "$UD/logs/Zed.log" /shot/Zed.log 2>/dev/null || true
find "$UD/extensions/work" -name install.log -exec sh -c 'cp "$1" "/shot/install.$(basename $(dirname $1)).log"' _ {} \; 2>/dev/null || true

kill $ZED_PID 2>/dev/null || true
kill $SWAY_PID 2>/dev/null || true
wait 2>/dev/null || true
exit $rc
