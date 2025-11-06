test:
    cargo test
    cargo run --example struct
    cargo run --example ext

tokei:
    tokei derive src --exclude src/tests.rs --exclude src/tests/

clippy:
    cargo clippy --fix --lib --allow-dirty --allow-staged
    cd derive && cargo clippy --fix --lib --allow-dirty --allow-staged
