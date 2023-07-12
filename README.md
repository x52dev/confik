# `confik`

[![crates.io](https://img.shields.io/crates/v/confik.svg)](http://crates.io/crates/confik)
[![docs.rs](https://docs.rs/confik/badge.svg)](http://docs.rs/confik)

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
use confik::{Configuration, EnvSource, FileSource, TomlSource};

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
