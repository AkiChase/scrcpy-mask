#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FFMPEG_VERSION="${FFMPEG_VERSION:-7.1.2}"
FFMPEG_DIR="$PROJECT_DIR/ffmpeg-$FFMPEG_VERSION"
TARGET_OS="${SCRCPY_MASK_FFMPEG_TARGET:-}"

if [[ -z "$TARGET_OS" ]]; then
    case "$(uname -s):$(uname -m)" in
        Darwin:arm64)
            TARGET_OS="macos-arm64"
            ;;
        Linux:x86_64)
            TARGET_OS="linux-x64"
            ;;
        MINGW64*:x86_64|MSYS_NT*:x86_64|CYGWIN_NT*:x86_64)
            TARGET_OS="windows-x64"
            ;;
        *)
            echo "Unsupported FFmpeg build target: $(uname -s) $(uname -m)" >&2
            exit 1
            ;;
    esac
fi

if [[ ! -d "$FFMPEG_DIR" ]]; then
    ARCHIVE="$PROJECT_DIR/FFmpeg-n$FFMPEG_VERSION.tar.gz"
    curl -L -o "$ARCHIVE" "https://github.com/FFmpeg/FFmpeg/archive/refs/tags/n$FFMPEG_VERSION.tar.gz"
    tar -xzf "$ARCHIVE" -C "$PROJECT_DIR"
    rm "$ARCHIVE"
    mv "$PROJECT_DIR/FFmpeg-n$FFMPEG_VERSION" "$FFMPEG_DIR"
fi

PREFIX="$FFMPEG_DIR/ffmpeg-$TARGET_OS"
CONFIGURE_ARGS=(
    "--prefix=$PREFIX"
    "--disable-all"
    "--disable-autodetect"
    "--disable-doc"
    "--disable-iconv"
    "--disable-network"
    "--disable-programs"
    "--enable-static"
    "--disable-shared"
    "--enable-pic"
    "--enable-avcodec"
    "--enable-avformat"
    "--enable-avutil"
    "--enable-swresample"
    "--enable-decoder=h264"
    "--enable-decoder=hevc"
    "--enable-decoder=av1"
    "--enable-decoder=opus"
    "--enable-decoder=aac"
    "--enable-decoder=flac"
)

if [[ "$TARGET_OS" == "windows-x64" ]]; then
    CONFIGURE_ARGS+=("--toolchain=msvc")
fi

if command -v nproc >/dev/null 2>&1; then
    JOBS="$(nproc)"
else
    JOBS="$(sysctl -n hw.ncpu)"
fi

echo "Building FFmpeg $FFMPEG_VERSION for $TARGET_OS"
cd "$FFMPEG_DIR"
make distclean >/dev/null 2>&1 || true
./configure "${CONFIGURE_ARGS[@]}"
make -j"$JOBS"
rm -rf "$PREFIX"
make install
rm -rf "$PREFIX/share"

echo "FFmpeg installed to: $PREFIX"
