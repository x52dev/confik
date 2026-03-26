_list:
    @just --list

# Lint workspace with Clippy
clippy:
    cargo clippy --workspace --no-default-features
    cargo clippy --workspace --all-features
    cargo hack --feature-powerset --depth=3 \
        --include-features env,json,ron-0_12,toml,yaml_serde-0_10,reloading,signal,tracing \
        clippy -p confik
    cargo hack --each-feature \
        --include-features ahash,bigdecimal,bytesize,camino,chrono,humantime,jiff-0_2,ipnetwork,js_option,rust_decimal,secrecy,serde_json,url,uuid \
        --exclude-features default \
        clippy -p confik

msrv := ```
    cargo metadata --format-version=1 \
    | jq -r 'first(.packages[] | select(.source == null and .rust_version)) | .rust_version' \
    | sed -E 's/^1\.([0-9]{2})$/1\.\1\.0/'
```
msrv_rustup := "+" + msrv

# Downgrade dev-dependencies necessary to run MSRV checks/tests.
[private]
downgrade-for-msrv toolchain="":
    cargo {{ toolchain }} update -p=serde_with --precise=3.16.1 # next ver: 1.82
    cargo {{ toolchain }} update -p=uuid --precise=1.20.0 # next ver: 1.85
    cargo {{ toolchain }} update -p=getrandom@0.4 --precise=0.3.4 # next ver: 1.85
    cargo {{ toolchain }} update -p=time --precise=0.3.45 # next ver: 1.88
    cargo {{ toolchain }} update -p=idna_adapter --precise=1.2.0 # next ver: 1.82
    cargo {{ toolchain }} update -p=proc-macro-crate --precise=3.4.0 # next ver: 1.82.0
    cargo {{ toolchain }} update -p=toml --precise=1.0.6+spec-1.1.0 # next ver: 1.85
    cargo {{ toolchain }} update -p=serde_spanned --precise=1.0.4 # next ver: 1.85
    cargo {{ toolchain }} update -p=toml_parser@1.1.0+spec-1.1.0 --precise=1.0.10+spec-1.1.0 # next ver: 1.85
    cargo {{ toolchain }} update -p=toml_writer@1.1.0+spec-1.1.0 --precise=1.0.7+spec-1.1.0 # next ver: 1.85
    cargo {{ toolchain }} update -p=toml_datetime@1.1.0+spec-1.1.0 --precise=1.0.1+spec-1.1.0 # next ver: 1.85
    cargo {{ toolchain }} update -p=yaml_serde --precise=0.10.2 # next ver: 1.82
    cargo {{ toolchain }} update -p=indexmap@2 --precise=2.11.4 # next ver: 1.82

# Test workspace using MSRV
test-msrv: (downgrade-for-msrv msrv_rustup) (test-no-coverage msrv_rustup)

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
    cargo clippy --workspace --all-features

# Format project
fmt:
    just --unstable --fmt
    nixpkgs-fmt .
    fd --type=file --hidden --extension=md --extension=yml --exec-batch prettier --write
    fd --hidden --extension=toml --exec-batch taplo format
    cargo +nightly fmt
