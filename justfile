default:
    just --list

test:
    cargo nextest run
    cargo test --doc
