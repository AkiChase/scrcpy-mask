#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
. "$SCRIPT_DIR/ffmpeg-env.sh"

if [[ "$SCRCPY_MASK_OS" != "linux-x64" ]]; then
    echo "scripts/package-linux.sh only supports linux-x64" >&2
    exit 1
fi

(cd "$PROJECT_DIR/frontend" && pnpm build)
(cd "$PROJECT_DIR" && cargo build --release)

BUNDLE_DIR="$PROJECT_DIR/target/release/tmp"
ASSETS_DIR="$PROJECT_DIR/assets"
BUILD_TARGET="$PROJECT_DIR/target/release/scrcpy-mask"
OUTPUT_ZIP="$PROJECT_DIR/target/release/scrcpy-mask-$SCRCPY_MASK_OS.zip"

rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"
cp -R "$ASSETS_DIR" "$BUNDLE_DIR/"
cp "$BUILD_TARGET" "$BUNDLE_DIR"

cat > "$BUNDLE_DIR/run.sh" <<'EOF'
#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$SCRIPT_DIR/scrcpy-mask" "$@"
EOF
chmod +x "$BUNDLE_DIR/run.sh"

rm -f "$OUTPUT_ZIP"
(cd "$BUNDLE_DIR" && zip -r "$OUTPUT_ZIP" ./*)
rm -rf "$BUNDLE_DIR"

echo "Zip created: $OUTPUT_ZIP"
