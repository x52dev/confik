_list:
    @just --list

clippy:
    cargo clippy --workspace --no-default-features
    cargo clippy --workspace --all-features
    cargo hack --feature-powerset --depth=3 clippy --workspace

test-msrv:
    @just test +1.66.0

test toolchain="":
    cargo {{toolchain}} test --package=confik-macros
    cargo {{toolchain}} test --package=confik --no-default-features
    @just test-coverage-codecov {{toolchain}}
    @just test-coverage-lcov {{toolchain}}
    RUSTDOCFLAGS="-D warnings" cargo {{toolchain}} doc --workspace --no-deps --all-features

test-coverage-codecov toolchain="":
    cargo {{toolchain}} llvm-cov --workspace --all-features --codecov --output-path codecov.json

test-coverage-lcov toolchain="":
    cargo {{toolchain}} llvm-cov --workspace --all-features --lcov --output-path lcov.info

doc:
    RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc --no-deps --workspace --all-features

doc-watch:
    RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc --no-deps --workspace --all-features --open
    cargo watch -- RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc --no-deps --workspace --all-features

check:
    just --unstable --fmt --check
    npx -y prettier --check '**/*.md'
    taplo lint
    cargo +nightly fmt -- --check

fmt:
    just --unstable --fmt
    npx -y prettier --write '**/*.md'
    taplo format
    cargo +nightly fmt
