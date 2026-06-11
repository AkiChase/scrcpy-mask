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

## Build FFMpeg

### Windows

Please use [MYSYS2](https://www.msys2.org/docs/environments/) and [MSVC](https://learn.microsoft.com/zh-cn/cpp/windows/latest-supported-vc-redist?view=msvc-170) for compilation.

```pwsh
.\scripts\build-ffmpeg.ps1
```

### macOS and Linux

```bash
./scripts/build-ffmpeg.sh
```


### Note:

- The script downloads FFmpeg when `ffmpeg-7.1.2` is missing.
- The build is static and only enables `avcodec`, `avformat`, `avutil`, and the H.264, H.265, and AV1 decoders.
- Set `FFMPEG_VERSION` to build a different FFmpeg release tag, for example `FFMPEG_VERSION=7.1.2 ./scripts/build-ffmpeg.sh`.

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
