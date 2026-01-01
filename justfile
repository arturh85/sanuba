start:
    RUST_LOG=info cargo run --release -- --regenerate

load:
    RUST_LOG=info cargo run --release

test:
    cargo fmt --all
    cargo fmt --all -- --check
    cargo clippy --fix --lib -p sunaba --tests --allow-dirty
    cargo test
    cargo build --release
    ./build-web.sh

web:
    ./build-web.sh
    cd web && python3 -m http.server 8080
