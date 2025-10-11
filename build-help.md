## Guild

This guide provides a brief description of how to run and compile the project.

## Web Frontend

Use [pnpm](https://pnpm.io/) to manage dependencies:

```bash
cd frontend
pnpm install
pnpm build
```

The build output will be in `assets/web`.

## FFMpeg

Since the project relies on FFmpeg, some additional steps are required to ensure FFmpeg is properly set up and available.

> I'm not familiar with FFmpeg, so the instructions below reflect my current configure. If you have a better build configure, feel free to submit a PR!

[FFmpeg Compilation Guide](https://trac.ffmpeg.org/wiki/CompilationGuide)

## Download FFMpeg

``` bash
# cd path/to/scrcpy-mask
curl -L -o FFmpeg-n7.1.2.tar.gz https://github.com/FFmpeg/FFmpeg/archive/refs/tags/n7.1.2.tar.gz
tar -xzf FFmpeg-n7.1.2.tar.gz
rm FFmpeg-n7.1.2.tar.gz
mv FFmpeg-n7.1.2 ffmpeg-7.1.2
cd ffmpeg-7.1.2
```

## Build FFMpeg

### Windows

Please use [MYSYS2](https://www.msys2.org/docs/environments/) and [MSVC](https://learn.microsoft.com/zh-cn/cpp/windows/latest-supported-vc-redist?view=msvc-170) for compilation.

```bash
make clean 2>/dev/null || true

OS="windows-x64"
./configure --prefix=./ffmpeg-$OS \
    --disable-all --disable-doc --disable-iconv \
    --toolchain=msvc \
    --enable-decoder=h264 --enable-decoder=hevc --enable-decoder=av1 \
    --enable-swscale --enable-avformat --enable-avcodec --enable-avutil --enable-swresample \
    --enable-gpl --enable-static --disable-shared

make -j$(nproc)
rm -rf ./ffmpeg-$OS
make install
```

### macOS

```bash
make clean 2>/dev/null || true
OS="macos-arm64"
./configure --prefix=./ffmpeg-$OS \
    --disable-all --disable-doc --disable-iconv \
    --enable-decoder=h264 --enable-decoder=hevc --enable-decoder=av1 \
    --enable-swscale --enable-avformat --enable-avcodec --enable-avutil --enable-swresample \
    --enable-gpl --enable-static --disable-shared

make -j$(sysctl -n hw.ncpu)
rm -rf ./ffmpeg-$OS
make install
```

### Linux

```bash
make clean 2>/dev/null || true
OS="linux-x64"
./configure --prefix=./ffmpeg-$OS \
    --disable-all --disable-doc --disable-iconv \
    --enable-decoder=h264 --enable-decoder=hevc --enable-decoder=av1 \
    --enable-swscale --enable-avformat --enable-avcodec --enable-avutil --enable-swresample \
    --enable-gpl --enable-static --disable-shared

make -j$(nproc)
rm -rf ./ffmpeg-$OS
make install
```

### Note:

- The dynamic library filenames required at runtime can be different. Be sure to adjust the filenames as needed to match the system's expected format (e.g., `libavcodec.61.dylib` instead of `libavcodec.61.19.101.dylib`).
- If you encounter missing dynamic library errors at runtime, please manually place the required libraries in the appropriate directory (e,g., `/assets/lib/windows-x64/libwinpthread-1.dll`)

## Run

### Example for Windows

```pwsh
$PREFIX = "ffmpeg-windows-x64"
$SCRIPT_DIR = Get-Location
$env:PKG_CONFIG_PATH = "$SCRIPT_DIR\ffmpeg-7.1.2\$PREFIX\lib\pkgconfig"
$env:FFMPEG_DIR = "$SCRIPT_DIR\ffmpeg-7.1.2\$PREFIX"

cargo run
```

### Example for macOS

```bash
PREFIX="ffmpeg-macos-arm64"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PKG_CONFIG_PATH="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX/lib/pkgconfig"
export FFMPEG_DIR="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX"

cargo run
```

### Example for Linux

```bash
PREFIX="ffmpeg-linux-x64"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PKG_CONFIG_PATH="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX/lib/pkgconfig"
export FFMPEG_DIR="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX"

cargo run
```


### Note

To ensure that the `rust-analyzer` extension in VS Code can correctly locate the FFmpeg dependencies, add the following configuration to your `settings.json`:

```json
"rust-analyzer.cargo.extraEnv": {
    "PKG_CONFIG_PATH": "/path/to/scrcpy-mask/ffmpeg-7.1.2/ffmpeg-macos-arm64/lib/pkgconfig",
    "FFMPEG_DIR": "/path/to/scrcpy-mask/ffmpeg-7.1.2/ffmpeg-macos-arm64"
}
```

Make sure to replace `/path/to/scrcpy-mask/` and `ffmpeg-macos-arm64` with the actual path to your local FFmpeg build directory. This configuration sets the necessary environment variables so that Cargo and rust-analyzer can find the FFmpeg libraries and headers during build and analysis.