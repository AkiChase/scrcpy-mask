# scrcpy-mask 常用开发指令

# list all recipes
default:
    @just --list

# start frontend dev server (hot-reload)
web-dev:
    cd frontend && pnpm dev

# build frontend (typecheck + vite build)
web-build:
    cd frontend && pnpm build

# build the app package
build:
    @if [ "{{os()}}" = "windows" ]; then \
        powershell -NoProfile -File scripts/package-windows.ps1; \
    elif [ "{{os()}}" = "macos" ]; then \
        ./scripts/package-macos.sh; \
    elif [ "{{os()}}" = "linux" ]; then \
        ./scripts/package-linux.sh; \
    else \
        echo "Unsupported OS: {{os()}}" >&2; exit 1; \
    fi

# update version, commit it, and create a release tag
release-version version:
    node scripts/release-version.mjs "{{version}}"

# run the app
run:
    @if [ "{{os()}}" = "windows" ]; then \
        powershell -NoProfile -File scripts/run-windows.ps1; \
    else \
        ./scripts/run.sh; \
    fi

# verify Rust compile, frontend typecheck + build, and lint
check:
    @if [ "{{os()}}" = "windows" ]; then \
        powershell -NoProfile -File scripts/check-windows.ps1; \
    else \
        ./scripts/check.sh; \
    fi
    cd frontend && pnpm lint
