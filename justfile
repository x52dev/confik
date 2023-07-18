_list:
    @just --list

clippy:
    cargo clippy --workspace --no-default-features
    cargo clippy --workspace --no-default-features --all-features
    cargo hack --feature-powerset --depth=3 clippy --workspace

test:
    cargo test --package=confik-macros
    cargo test --package=confik --no-default-features
    cargo test --package=confik --no-default-features --all-features

check:
    just --unstable --fmt --check
    npx -y prettier --check '**/*.md'
    taplo lint

fmt:
    just --unstable --fmt
    npx -y prettier --write '**/*.md'
    taplo format
