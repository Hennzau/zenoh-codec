test:
    cargo test

bench:
    cargo test --profile=release bench -- --nocapture --ignored
