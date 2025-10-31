test:
    cargo test

tokei:
    tokei derive src --exclude src/tests.rs --exclude src/tests/

clippy:
    cargo clippy --fix --lib --allow-dirty --allow-staged
    cd derive && cargo clippy --fix --lib --allow-dirty --allow-staged
