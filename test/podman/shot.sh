#!/bin/sh
# shot.sh WORKSPACE FILE OUT.png [DEV_EXT_DIR ...]
# Start Zed under headless sway in the zed-shot image, open FILE, wait,
# then save a screenshot and the Zed logs next to OUT.png.
# Env: WAIT_SECS (default 12), SETTINGS_JSON (default: settings.json here),
#      ZED_SHOT_IMAGE (default zed-shot:latest), PODMAN_NETWORK (default slirp4netns).
set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
IMG=${ZED_SHOT_IMAGE:-zed-shot:latest}
WORKSPACE=${1:?workspace dir}
FILE=${2:?file to open, host path under WORKSPACE}
OUT=${3:?output png path on host}
shift 3

WORKSPACE=$(realpath "$WORKSPACE")
FILE=$(realpath "$FILE")
case "$OUT" in /*) OUTHOST="$OUT" ;; *) OUTHOST="$PWD/$OUT" ;; esac
mkdir -p "$(dirname "$OUTHOST")"

FILECONT="$FILE"
case "$FILE" in
  "$WORKSPACE"/*) FILECONT="/work/${FILE#"$WORKSPACE"/}" ;;
esac

VOLUMES="-v $WORKSPACE:/work"
for d in "$@"; do
  [ -d "$d" ] || { echo "dev ext missing: $d" >&2; exit 2; }
  VOLUMES="$VOLUMES -v $d:/dev-ext-in/$(basename "$d"):ro"
done

VOLUMES="$VOLUMES -v ${SETTINGS_JSON:-$HERE/settings.json}:/rig/settings.json:ro"
VOLUMES="$VOLUMES -v $HERE/in-container.sh:/rig/in-container.sh:ro"
OUTDIR=$(mktemp -d)
chmod 777 "$OUTDIR"
VOLUMES="$VOLUMES -v $OUTDIR:/shot"

# Zed gets killed at the end of in-container.sh, so a non-zero status is normal.
rc=0
# shellcheck disable=SC2086
podman run --rm -i \
  --network="${PODMAN_NETWORK:-slirp4netns}" \
  --userns=keep-id \
  --ipc=host \
  $VOLUMES \
  -e ZED_ALLOW_ROOT=1 \
  "$IMG" \
  sh /rig/in-container.sh "$FILECONT" "${WAIT_SECS:-12}" /rig/settings.json || rc=$?

BASE="${OUTHOST%.png}"
cp "$OUTDIR/shot.png" "$OUTHOST" 2>/dev/null || true
for n in zed.log sway.log extensions.txt Zed.log workdir.txt home-uv.txt index.json; do
  cp "$OUTDIR/$n" "$BASE.$n" 2>/dev/null || true
done
for f in "$OUTDIR"/install.*.log; do
  [ -f "$f" ] && cp "$f" "$BASE.$(basename "$f")"
done
rm -rf "$OUTDIR"
echo "podman exit=$rc"
