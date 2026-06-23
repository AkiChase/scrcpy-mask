#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_DIR/assets/platform-tools"

ADB_BIN="$(command -v adb || true)"
if [[ -z "$ADB_BIN" ]]; then
    echo "adb not found in PATH. Install Android SDK Platform-Tools before building." >&2
    exit 1
fi

resolve_path() {
    local path="$1"
    while [[ -L "$path" ]]; do
        local target
        target="$(readlink "$path")"
        if [[ "$target" == /* ]]; then
            path="$target"
        else
            path="$(dirname "$path")/$target"
        fi
    done

    local dir
    dir="$(cd "$(dirname "$path")" && pwd -P)"
    printf '%s/%s\n' "$dir" "$(basename "$path")"
}

ADB_BIN="$(resolve_path "$ADB_BIN")"
ADB_DIR="$(dirname "$ADB_BIN")"

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

cp "$ADB_BIN" "$OUTPUT_DIR/adb"
chmod +x "$OUTPUT_DIR/adb"

for file in NOTICE.txt source.properties; do
    if [[ -f "$ADB_DIR/$file" ]]; then
        cp "$ADB_DIR/$file" "$OUTPUT_DIR/"
    fi
done

echo "Bundled adb from $ADB_BIN"
