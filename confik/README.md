# `confik`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/confik?label=latest)](https://crates.io/crates/confik)
[![Documentation](https://docs.rs/confik/badge.svg?version=0.9.0)](https://docs.rs/confik/0.9.0)
![Version](https://img.shields.io/badge/rustc-1.65+-ab6000.svg)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/confik.svg)
<br />
[![CI](https://github.com/x52dev/confik/actions/workflows/ci.yml/badge.svg)](https://github.com/x52dev/confik/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/crate/confik/0.9.0/status.svg)](https://deps.rs/crate/confik/0.9.0)
[![Download](https://img.shields.io/crates/d/confik.svg)](https://crates.io/crates/confik)

<!-- prettier-ignore-end -->

This crate provides a macro for creating configuration/settings structures and functions to read them from files and the environment.

## Example

Assume that `config.toml` contains

```toml
host = "google.com"
username = "root"
```

and the environment contains

```sh
PASSWORD=hunter2
```

Then:

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
