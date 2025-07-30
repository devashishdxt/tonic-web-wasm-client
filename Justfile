# Builds `tonic-web-wasm-client` for `wasm32-unknown-unknown` target
build:
    @echo 'Building...'
    cargo build --target wasm32-unknown-unknown

# Builds `tonic-web-wasm-client` for `wasm32-wasip2` target
build-wasip2:
    @echo 'Building...'
    cargo build --target wasm32-wasip2

# Builds test `tonic-web` server natively
build-test-server:
    @echo 'Building test server...'
    cd test-suite/simple/server && cargo build

# Starts test `tonic-web` server
start-test-server:
    @echo 'Starting test server...'
    cd test-suite/simple/server && cargo run

# Runs browser tests for `tonic-web-wasm-client`
test:
    @echo 'Testing...'
    cd test-suite/simple/client && wasm-pack test --chrome

# Runs browser tests for `tonic-web-wasm-server` (in headless mode)
test-headless:
    @echo 'Testing...'
    cd test-suite/simple/client && wasm-pack test --headless --chrome

# Checks wasmtime version, needed only for -wasip2 recipes
check-wasmtime-version:
    #!/usr/bin/env bash
    set -euo pipefail
    version_output=$(wasmtime --version)
    current_version=$(echo "${version_output}" | awk '{print $2}')
    required_version="35.0.0"
    if ! printf '%s\n' "${required_version}" "${current_version}" | sort --check=silent --version-sort; then
        echo "Error: wasmtime version ${current_version} is installed."
        echo "Version 35.0.0 or greater is required."
        exit 1
    fi

# Runs wasip2 tests in `wasmtime`
test-wasip2: check-wasmtime-version
    cd test-suite/simple/client && cargo test --target wasm32-wasip2

# Builds test `tonic-web` server (with compression enabled: gzip) for `wasm32-unknown-unknown` target
build-gzip-test-server:
    @echo 'Building test server...'
    cd test-suite/gzip/server && cargo build

# Starts test `tonic-web` server (with compression enabled: gzip) natively
start-gzip-test-server:
    @echo 'Starting test server...'
    cd test-suite/gzip/server && cargo run

# Runs browser tests for `tonic-web-wasm-client` (with compression enabled: gzip)
test-gzip:
    @echo 'Testing...'
    cd test-suite/gzip/client && wasm-pack test --chrome

# Runs browser tests for `tonic-web-wasm-server` (in headless mode) (with compression enabled: gzip)
test-gzip-headless:
    @echo 'Testing...'
    cd test-suite/gzip/client && wasm-pack test --headless --chrome

# Runs wasip2 tests (with compression enabled: gzip) in `wasmtime`
test-gzip-wasip2: check-wasmtime-version
    @echo 'Testing...'
    cd test-suite/gzip/client &&  cargo test --target wasm32-wasip2

