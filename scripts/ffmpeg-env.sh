#!/usr/bin/env bash

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    echo "Source this script instead of executing it: . ./scripts/ffmpeg-env.sh" >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FFMPEG_VERSION="${FFMPEG_VERSION:-7.1.2}"

case "$(uname -s):$(uname -m)" in
    Darwin:arm64)
        SCRCPY_MASK_OS="macos-arm64"
        ;;
    Linux:x86_64)
        SCRCPY_MASK_OS="linux-x64"
        ;;
    *)
        echo "Unsupported FFmpeg target: $(uname -s) $(uname -m)" >&2
        return 1
        ;;
esac

export SCRCPY_MASK_OS
export FFMPEG_DIR="$PROJECT_DIR/ffmpeg-$FFMPEG_VERSION/ffmpeg-$SCRCPY_MASK_OS"
export PKG_CONFIG_PATH="$FFMPEG_DIR/lib/pkgconfig"

if [[ ! -d "$FFMPEG_DIR" ]]; then
    echo "FFmpeg not found at $FFMPEG_DIR. Run ./scripts/build-ffmpeg.sh first." >&2
    return 1
fi
