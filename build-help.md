## Guide

This guide describes the common setup, development, verification, and packaging flow.

## Prerequisites

- Rust toolchain
- [just](https://github.com/casey/just)
- [pnpm](https://pnpm.io/)
- Windows FFmpeg build: [MSYS2](https://www.msys2.org/docs/environments/) for the build shell, Visual Studio Build Tools with C++ build tools for the MSVC compiler, and `nasm` available in MSYS2

## First-time Setup

Run the setup recipe after cloning the repository:

```bash
just setup
```

This installs frontend dependencies with `pnpm install`, then builds the local FFmpeg static libraries.

On Windows, install the prerequisites before running `just setup`: MSYS2 for `bash`/Unix build tools, Visual Studio Build Tools for the MSVC compiler, and `nasm` from MSYS2. The script uses MSYS2 bash at `C:\msys64\usr\bin\bash.exe` by default; set `MSYS2_BASH` to the full `bash.exe` path if MSYS2 is installed elsewhere.

## FFmpeg

The project links against a local FFmpeg build. The setup and FFmpeg build recipes both use the existing platform-specific scripts:

```bash
just build-ffmpeg
```

The FFmpeg build script downloads the FFmpeg source archive when `ffmpeg-7.1.2` is missing, then builds and installs static libraries for the current target.

On Windows, `scripts/build-ffmpeg.ps1` checks for MSYS2 bash, loads the MSVC C++ build tools environment with `VsDevCmd.bat`, sets `MSYS2_PATH_TYPE=inherit`, and verifies that MSYS2 bash can resolve `cl.exe` before running FFmpeg configure. MSYS2 provides the shell used to run FFmpeg's build scripts; MSVC provides the compiler selected by `--toolchain=msvc`. The script does not fall back to an arbitrary `bash` executable.

Notes:

- The default FFmpeg version is `7.1.2`.
- The build only enables `avcodec`, `avformat`, `avutil`, and the H.264, H.265, and AV1 decoders.
- Set `FFMPEG_VERSION` to build a different FFmpeg release tag, for example `FFMPEG_VERSION=7.1.2 just build-ffmpeg`.

Windows troubleshooting:

- If FFmpeg reports `cl.exe is unable to create an executable file` and `config.log` contains `cl.exe: command not found`, MSYS2 did not inherit the Visual Studio environment. Use the current `scripts/build-ffmpeg.ps1`; it sets `MSYS2_PATH_TYPE=inherit` automatically.
- If `cl.exe` is still missing, install Visual Studio Build Tools with the C++ build tools workload, or run from a shell where `vswhere.exe` can find that installation.
- If `nasm` is missing, install it in MSYS2, for example `pacman -S nasm`, then rerun `just build-ffmpeg`.

## Development

Run the desktop app:

```bash
just run
```

Start the frontend dev server:

```bash
just web-dev
```

Build the frontend only:

```bash
just web-build
```

The frontend build output is written to `assets/web`.

## Verification

Run the standard project checks:

```bash
just check
```

`just check` loads the FFmpeg environment, runs `cargo check`, then runs the frontend linter. It does not rebuild FFmpeg.

## Packaging

Build the release package for the current platform:

```bash
just build
```

The package scripts build the frontend, build the Rust app in release mode, and create the platform package. They require FFmpeg to have been built already.

## Rust Analyzer

To ensure that `rust-analyzer` can locate FFmpeg dependencies, add the matching local FFmpeg paths to VS Code `settings.json`:

```json
"rust-analyzer.cargo.extraEnv": {
    "PKG_CONFIG_PATH": "/path/to/scrcpy-mask/ffmpeg-7.1.2/ffmpeg-macos-arm64/lib/pkgconfig",
    "FFMPEG_DIR": "/path/to/scrcpy-mask/ffmpeg-7.1.2/ffmpeg-macos-arm64"
}
```

Replace `/path/to/scrcpy-mask/` and `ffmpeg-macos-arm64` with the actual local path and target directory.
