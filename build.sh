#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FFMPEG="ffmpeg-7.1.2"

if [[ "$(uname)" == "Darwin" ]]; then
    echo "Building for MacOS arm64"
    PREFIX="ffmpeg-macos"
    OS="macos-arm64"
elif [[ "$(uname)" == "Linux" ]]; then
    echo "Building for Linux x64"
    PREFIX="ffmpeg-linux"
    OS="linux-x64"
else
    echo "Unhandled system: $(uname). Exiting."
    exit 1
fi

export PKG_CONFIG_PATH="$SCRIPT_DIR/$FFMPEG/$PREFIX/lib/pkgconfig"
export FFMPEG_DIR="$SCRIPT_DIR/$FFMPEG/$PREFIX"
export DYLD_LIBRARY_PATH="$SCRIPT_DIR/assets/lib/$OS:$DYLD_LIBRARY_PATH"

if [[ "$1" == "run" ]]; then
    cargo run
    exit $?
elif [[ "$1" == "release" ]]; then
    cd "$SCRIPT_DIR/frontend"
    pnpm build
    if [[ $? -ne 0 ]]; then
        echo "Frontend build failed"
        exit 1
    fi

    cd "$SCRIPT_DIR"
    cargo build --release
    if [[ $? -ne 0 ]]; then
        echo "Project build failed"
        exit 1
    fi

    ASSETS_DIR="$SCRIPT_DIR/assets"
    LIB_OS_FOLDER="$ASSETS_DIR/lib/$OS"

    if [[ "$(uname)" == "Darwin" ]]; then
        export CARGO_BUNDLE_SKIP_BUILD="1"
        cargo bundle -r

        echo "Adjusting bundle files..."
        BUNDLE_DIR="$SCRIPT_DIR/target/release/bundle/osx/scrcpy-mask.app"
        DMG_PATH="$SCRIPT_DIR/target/release/scrcpy-mask.dmg"
        APP_BIN_DIR="$BUNDLE_DIR/Contents/MacOS"
        mv "$APP_BIN_DIR/scrcpy-mask" "$APP_BIN_DIR/scrcpy-mask-bin"
cat > "$APP_BIN_DIR/scrcpy-mask" << 'EOF'
#!/bin/bash

APP_DIR="$(cd "$(dirname "$0")" && pwd)"

CMD="cd $APP_DIR && ./scrcpy-mask-bin; echo 'Done. Press any key to exit...'; read"

osascript -e "tell application \"Terminal\" to do script \"$CMD\""
osascript -e "tell application \"Terminal\" to activate"
EOF
        chmod +x "$APP_BIN_DIR/scrcpy-mask"

        BUNDLE_ASSETS_DIR="$APP_BIN_DIR/assets"
        BUNDLE_LIB_DIR="$APP_BIN_DIR/ffmpeg-macos/lib"

        if [[ ! -d "$LIB_OS_FOLDER" ]]; then
            echo "Required folder not found: $LIB_OS_FOLDER"
            exit 1
        fi
        mkdir -p "$BUNDLE_ASSETS_DIR"
        mkdir -p "$BUNDLE_LIB_DIR"
        
        find "$ASSETS_DIR" -mindepth 1 -maxdepth 1 ! -name 'lib' -exec cp -R {} "$BUNDLE_ASSETS_DIR" \;
        find "$LIB_OS_FOLDER" -maxdepth 1 -type f -exec cp '{}' "$BUNDLE_LIB_DIR/" \;

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
        exit $?
    elif [[ "$(uname)" == "Linux" ]]; then
        BUNDLE_DIR="$SCRIPT_DIR/target/release/tmp"
        BUNDLE_ASSETS_DIR="$BUNDLE_DIR/assets"
        BUNDLE_LIB_DIR="$BUNDLE_DIR/ffmpeg-linux/lib"
        if [[ ! -d "$LIB_OS_FOLDER" ]]; then
            echo "Required folder not found: $LIB_OS_FOLDER"
            exit 1
        fi
        mkdir -p "$BUNDLE_ASSETS_DIR"
        mkdir -p "$BUNDLE_LIB_DIR"

        find "$ASSETS_DIR" -mindepth 1 -maxdepth 1 ! -name 'lib' -exec cp -R {} "$BUNDLE_ASSETS_DIR" \;
        find "$LIB_OS_FOLDER" -maxdepth 1 -type f -exec cp '{}' "$BUNDLE_LIB_DIR/" \;
        BUILD_TARGET="$SCRIPT_DIR/target/release/scrcpy-mask"
        cp "$BUILD_TARGET" "$BUNDLE_DIR"

        OUTPUT_ZIP="$SCRIPT_DIR/target/release/scrcpy-mask-$OS.zip"
        rm -f "$OUTPUT_ZIP"
        
        cd "$BUNDLE_DIR"
        zip -r "$OUTPUT_ZIP" ./*
        rm -rf "$BUNDLE_DIR"
        cd "$SCRIPT_DIR"
        
        echo "Zip created: $OUTPUT_ZIP"
        exit $?
    else
        echo "Unhandled system: $(uname). Exiting."
        exit 1
    fi
else
    echo "Usage: $0 {run|release}"
    exit 1
fi
