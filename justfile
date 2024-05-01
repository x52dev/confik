_list:
    @just --list

clippy:
    cargo clippy --workspace --no-default-features
    cargo clippy --workspace --all-features
    cargo hack --feature-powerset --depth=3 clippy --workspace

# Downgrade dev-dependencies necessary to run MSRV checks/tests.
[private]
downgrade-msrv:
    cargo update -p=toml --precise=0.8.8
    cargo update -p=toml_edit --precise=0.21.0
    cargo update -p=trybuild --precise=1.0.90

# Test workspace using MSRV
test-msrv: downgrade-msrv
    @just test-no-coverage +1.67.0

# Test workspace without generating coverage files
[private]
test-no-coverage toolchain="":
    cargo {{ toolchain }} test --lib --tests --package=confik-macros
    cargo {{ toolchain }} nextest run --package=confik --no-default-features
    cargo {{ toolchain }} nextest run --package=confik --all-features
    cargo {{ toolchain }} test --doc --workspace --all-features
    RUSTDOCFLAGS="-D warnings" cargo {{ toolchain }} doc --workspace --no-deps --all-features

# Test workspace and generate coverage files
test toolchain="": (test-no-coverage toolchain)
    @just test-coverage-codecov {{ toolchain }}
    @just test-coverage-lcov {{ toolchain }}

test-coverage-codecov toolchain="":
    cargo {{ toolchain }} llvm-cov --workspace --all-features --codecov --output-path codecov.json

test-coverage-lcov toolchain="":
    cargo {{ toolchain }} llvm-cov --workspace --all-features --lcov --output-path lcov.info

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
