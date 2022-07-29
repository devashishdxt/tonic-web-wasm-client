# Builds `tonic-web-wasm-client`
build:
    @echo 'Building...'
    cargo build --target wasm32-unknown-unknown

# Starts test `tonic-web` server
start-test-server:
    @echo 'Starting test server...'
    cd test-suite/server && cargo run

# Runs browser tests for `tonic-web-wasm-client`
test:
    @echo 'Testing...'
    cd test-suite/client && wasm-pack test --chrome

# Runs browser tests for `tonic-web-wasm-server` (in headless mode)
test-headless:
    @echo 'Testing...'
    cd test-suite/client && wasm-pack test --headless --chrome
