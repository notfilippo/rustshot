#!/bin/bash

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

for SIZE in 32 128 512
do
  echo "Generating ${SIZE}x${SIZE}"
  SIZE_2X=$((SIZE * 2))
  rsvg-convert -h "$SIZE" "$SCRIPT_DIR/camera.svg" > "$SCRIPT_DIR/icon-${SIZE}x${SIZE}.png"
  rsvg-convert -h "$SIZE_2X" "$SCRIPT_DIR/camera.svg" > "$SCRIPT_DIR/icon-${SIZE}x${SIZE}@2x.png"
done

rsvg-convert -h 128 "$SCRIPT_DIR/camera.svg" > "$SCRIPT_DIR/icon.png"
rsvg-convert -h 128 "$SCRIPT_DIR/camera.svg" > "$SCRIPT_DIR/icon@2x.png"