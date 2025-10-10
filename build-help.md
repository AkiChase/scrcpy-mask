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
wget https://ffmpeg.org/releases/ffmpeg-7.1.2.tar.bz2
tar -xjf ffmpeg-7.1.2.tar.bz2
rm ffmpeg-7.1.2.tar.bz2
cd ffmpeg-7.1.2
```

## Build FFMpeg

```bash
OS="windows"
# OS="macos"
# OS="linux"

./configure --prefix=./ffmpeg-$OS \
    --disable-avdevice --disable-postproc \
    --enable-decoder=h264 --enable-decoder=hevc --enable-decoder=av1 \
    --enable-swscale --enable-filter=scale \
    --enable-avformat --enable-avcodec --enable-avutil --enable-swresample \
    --enable-gpl --disable-static --enable-shared

make -j$(nproc)
# For macOS, use the following command
# make -j$(sysctl -n hw.ncpu)

make install
```

## Copy Dynamic Library

After a successful compilation, copy the corresponding dynamic link libraries to the appropriate directory under `/assets/lib/<system>` (refer to the directory structure).

For example, if you're targeting Windows, copy the `.dll` files; for macOS, copy the `.dylib` files; and for Linux, copy the `.so` files. This ensures that the system can find and link to the required libraries at runtime.

```bash
# Example for copying dynamic libraries (adjust paths as necessary)
cp /path/to/compiled/libs/*.so.* /assets/lib/linux-x64/
cp /path/to/compiled/libs/*.dylib /assets/lib/darwin-arm64/
cp /path/to/compiled/libs/*.dll /assets/lib/windows-x64/
```

### Note:

- The dynamic library filenames required at runtime can be different. Be sure to adjust the filenames as needed to match the system's expected format (e.g., `libavcodec.61.dylib` instead of `libavcodec.61.19.101.dylib`).
- If you encounter missing dynamic library errors at runtime, please manually place the required libraries in the appropriate directory (e,g., `/assets/lib/windows-x64/libwinpthread-1.dll`)

## Run

### Example for Windows

```pwsh
$PREFIX = "ffmpeg-windows"
$DYLIB = "windows-x64"
$SCRIPT_DIR = Get-Location

$env:PKG_CONFIG_PATH = "$SCRIPT_DIR\ffmpeg-7.1.2\$PREFIX\lib\pkgconfig"
$env:FFMPEG_DIR = "$SCRIPT_DIR\ffmpeg-7.1.2\$PREFIX"
$env:PATH = "$SCRIPT_DIR\assets\lib\$DYLIB;$env:PATH"

cargo run
```

### Example for macOS

```bash
PREFIX="ffmpeg-macos"
DYLIB="macos-arm64"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export PKG_CONFIG_PATH="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX/lib/pkgconfig"
export FFMPEG_DIR="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX"
export DYLD_LIBRARY_PATH="$SCRIPT_DIR/assets/lib/$DYLIB:$DYLD_LIBRARY_PATH"

cargo run
```

### Example for Linux

```bash
PREFIX="ffmpeg-linux"
DYLIB="linux-x64"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export PKG_CONFIG_PATH="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX/lib/pkgconfig"
export FFMPEG_DIR="$SCRIPT_DIR/ffmpeg-7.1.2/$PREFIX"
export LD_LIBRARY_PATH="$SCRIPT_DIR/assets/lib/$DYLIB:$LD_LIBRARY_PATH"

cargo run
```


### Note

To ensure that the `rust-analyzer` extension in VS Code can correctly locate the FFmpeg dependencies, add the following configuration to your `settings.json`:

```json
"rust-analyzer.cargo.extraEnv": {
    "PKG_CONFIG_PATH": "/path/to/scrcpy-mask/ffmpeg-7.1.2/ffmpeg-macos/lib/pkgconfig",
    "FFMPEG_DIR": "/path/to/scrcpy-mask/ffmpeg-7.1.2/ffmpeg-macos"
}
```

Make sure to replace `/path/to/scrcpy-mask/` with the actual path to your local FFmpeg build directory. This configuration sets the necessary environment variables so that Cargo and rust-analyzer can find the FFmpeg libraries and headers during build and analysis.