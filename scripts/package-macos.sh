#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
. "$SCRIPT_DIR/ffmpeg-env.sh"

if [[ "$SCRCPY_MASK_OS" != "macos-arm64" ]]; then
    echo "scripts/package-macos.sh only supports macos-arm64" >&2
    exit 1
fi

"$SCRIPT_DIR/prepare-adb.sh"
(cd "$PROJECT_DIR/frontend" && pnpm build)
(cd "$PROJECT_DIR" && cargo build --release)

export CARGO_BUNDLE_SKIP_BUILD="1"
(cd "$PROJECT_DIR" && cargo bundle -r)

ASSETS_DIR="$PROJECT_DIR/assets"
BUNDLE_DIR="$PROJECT_DIR/target/release/bundle/osx/scrcpy-mask.app"
DMG_PATH="$PROJECT_DIR/target/release/scrcpy-mask-$SCRCPY_MASK_OS.dmg"
APP_BIN_DIR="$BUNDLE_DIR/Contents/MacOS"

echo "Adjusting bundle files..."
mv "$APP_BIN_DIR/scrcpy-mask" "$APP_BIN_DIR/scrcpy-mask-bin"
cat > "$APP_BIN_DIR/scrcpy-mask" <<'EOF'
#!/bin/bash

APP_DIR="$(cd "$(dirname "$0")" && pwd)"
printf -v CMD "cd %q && ./scrcpy-mask-bin" "$APP_DIR"

TERMINAL_WAS_RUNNING=0
if pgrep -qx Terminal >/dev/null 2>&1; then
    TERMINAL_WAS_RUNNING=1
fi

osascript \
    -e 'on run argv' \
    -e 'set shellCommand to item 1 of argv' \
    -e 'set terminalRunning to (item 2 of argv is "1")' \
    -e 'tell application "Terminal"' \
    -e 'if terminalRunning then' \
    -e 'do script shellCommand' \
    -e 'else' \
    -e 'activate' \
    -e 'if (count of windows) is 0 then' \
    -e 'do script shellCommand' \
    -e 'else' \
    -e 'do script shellCommand in front window' \
    -e 'end if' \
    -e 'end if' \
    -e 'activate' \
    -e 'end tell' \
    -e 'end run' \
    "$CMD" "$TERMINAL_WAS_RUNNING"
EOF
chmod +x "$APP_BIN_DIR/scrcpy-mask"
rm -rf "$APP_BIN_DIR/assets"
cp -R "$ASSETS_DIR" "$APP_BIN_DIR/"

rm -f "$DMG_PATH"
create-dmg \
    --volname "scrcpy-mask" \
    --volicon "./icons/icon.icns" \
    --window-pos 200 120 \
    --window-size 600 300 \
    --icon "scrcpy-mask.app" 150 100 \
    --app-drop-link 450 100 \
    "$DMG_PATH" "$BUNDLE_DIR"

echo "DMG created: $DMG_PATH"
