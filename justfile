# scrcpy-mask 常用开发指令

# start frontend dev server (hot-reload)
web-dev:
    cd frontend && pnpm dev

# build frontend (typecheck + vite build)
web-build:
    cd frontend && pnpm build

# run the app via build script
run:
    @if [ "{{os()}}" = "windows" ]; then \
        powershell -File build.ps1 run; \
    else \
        ./build.sh run; \
    fi

# verify Rust compile, frontend typecheck + build, and lint
check:
    cargo check
    cd frontend && pnpm lint
