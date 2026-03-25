# `confik`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/confik?label=latest)](https://crates.io/crates/confik)
[![Documentation](https://docs.rs/confik/badge.svg?version=0.15.8)](https://docs.rs/confik/0.15.8)
[![dependency status](https://deps.rs/crate/confik/0.15.8/status.svg)](https://deps.rs/crate/confik/0.15.8)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/confik.svg)
<br />
[![CI](https://github.com/x52dev/confik/actions/workflows/ci.yml/badge.svg)](https://github.com/x52dev/confik/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/x52dev/confik/branch/main/graph/badge.svg)](https://codecov.io/gh/x52dev/confik)
![Version](https://img.shields.io/badge/rustc-1.65+-ab6000.svg)
[![Download](https://img.shields.io/crates/d/confik.svg)](https://crates.io/crates/confik)

<!-- prettier-ignore-end -->

`confik` is a configuration library for Rust applications that need to compose settings from multiple sources without giving up type safety.

It is built for the common production path: read defaults from code, layer in config files, override with environment variables, keep secrets out of insecure sources, and build one strongly typed config value for the rest of your application.

## Built for Real App Config

- **Derive-first API** -- define your config once and get a builder that merges partial values from many sources.
- **Multi-source by design** -- combine files, environment variables, and inline formats in a predictable override order.
- **Secret-aware loading** -- mark sensitive fields and opt into reading them only from trusted sources.
- **Production-friendly features** -- support hot reloading and SIGHUP-triggered refreshes when your application needs them.
- **Serde ecosystem compatibility** -- reuse familiar `serde` attributes and common third-party config value types.

## Example

Assume your application ships with a `config.toml` file:

```toml
host = "google.com"
username = "root"
```

and your deployment injects the secret through the environment:

```sh
PASSWORD=hunter2
```

Then `confik` can merge both into one typed config object:

```rust
use confik::{Configuration, EnvSource, FileSource};

#[derive(Debug, PartialEq, Configuration)]
struct Config {
    host: String,
    username: String,

    #[confik(secret)]
    password: String,
}

fn main() {
    let config = Config::builder()
        .override_with(FileSource::new("config.toml"))
        .override_with(EnvSource::new().allow_secrets())
        .try_build()
        .unwrap();

    assert_eq!(
        config,
        Config {
            host: "google.com".to_string(),
            username: "root".to_string(),
            password: "hunter2".to_string(),
        }
    );
}
```

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.
