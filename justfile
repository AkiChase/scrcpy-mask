# scrcpy-mask 常用开发指令

set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

build_ffmpeg_cmd := if os() == "windows" { "powershell -NoProfile -File scripts/build-ffmpeg.ps1" } else { "./scripts/build-ffmpeg.sh" }
build_package_cmd := if os() == "windows" { "powershell -NoProfile -File scripts/package-windows.ps1" } else if os() == "macos" { "./scripts/package-macos.sh" } else { "./scripts/package-linux.sh" }
run_cmd := if os() == "windows" { "powershell -NoProfile -File scripts/run-windows.ps1" } else { "./scripts/run.sh" }
check_cmd := if os() == "windows" { "powershell -NoProfile -File scripts/check-windows.ps1" } else { "./scripts/check.sh" }

# list all recipes
default:
    @just --list

# install frontend dependencies and build FFmpeg
setup:
    pnpm --dir frontend install
    just build-ffmpeg

# start frontend dev server (hot-reload)
web-dev:
    pnpm --dir frontend dev

# build frontend (typecheck + vite build)
web-build:
    pnpm --dir frontend build

# build FFmpeg static libraries
build-ffmpeg:
    {{build_ffmpeg_cmd}}

# build the app package
build:
    {{build_package_cmd}}

# update version, commit it, and create a release tag
release-version version:
    node scripts/release-version.mjs "{{version}}"

# run the app
run:
    {{run_cmd}}

# verify Rust compile and frontend lint
check:
    {{check_cmd}}
    pnpm --dir frontend lint
