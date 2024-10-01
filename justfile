_list:
    @just --list

# Lint workspace with Clippy
clippy:
    cargo clippy --workspace --no-default-features
    cargo clippy --workspace --no-default-features --all-features
    cargo hack --feature-powerset --depth=3 clippy --workspace

msrv := ```
    cargo metadata --format-version=1 \
    | jq -r 'first(.packages[] | select(.source == null and .rust_version)) | .rust_version' \
    | sed -E 's/^1\.([0-9]{2})$/1\.\1\.0/'
```
msrv_rustup := "+" + msrv

# Downgrade dev-dependencies necessary to run MSRV checks/tests.
[private]
downgrade-msrv:
    cargo update -p=trybuild --precise=1.0.90

# Test workspace using MSRV
test-msrv: downgrade-msrv (test-no-coverage msrv_rustup)

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

# Test workspace and generate Codecov coverage file
test-coverage-codecov toolchain="":
    cargo {{ toolchain }} llvm-cov --workspace --all-features --codecov --output-path codecov.json

# Test workspace and generate LCOV coverage file
test-coverage-lcov toolchain="":
    cargo {{ toolchain }} llvm-cov --workspace --all-features --lcov --output-path lcov.info

# Document workspace
doc:
    RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc --no-deps --workspace --all-features

# Document workspace and watch for changes
doc-watch:
    RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc --no-deps --workspace --all-features --open
    cargo watch -- RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc --no-deps --workspace --all-features

# Check project
check:
    just --unstable --fmt --check
    nixpkgs-fmt --check .
    fd --type=file --hidden --extension=md --extension=yml --exec-batch prettier --check
    fd --hidden --extension=toml --exec-batch taplo format --check
    fd --hidden --extension=toml --exec-batch taplo lint
    cargo +nightly fmt -- --check

# Format project
fmt:
    just --unstable --fmt
    nixpkgs-fmt .
    fd --type=file --hidden --extension=md --extension=yml --exec-batch prettier --write
    fd --hidden --extension=toml --exec-batch taplo format
    cargo +nightly fmt
