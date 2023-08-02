# `confik`

`confik` is a library for reading application configuration split across multiple sources.

## Example

Assume that `config.toml` contains:

```toml
host=google.com
username=root
```

and the environment contains:

```bash
PASSWORD=hunter2
```

then:

```no_run
# #[cfg(all(feature = "toml", feature = "env"))]
# {
use confik::{Configuration, EnvSource, FileSource, TomlSource};

#[derive(Debug, PartialEq, Configuration)]
struct Config {
    host: String,
    username: String,

    #[confik(secret)]
    password: String,
}

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
# }
```

## Sources

A [`Source`] is any type that can create [`ConfigurationBuilder`]s. This crate implements the following sources:

- [`EnvSource`]: Loads configuration from environment variables using the [`envious`] crate. Requires the `env` feature. (Enabled by default.)
- [`FileSource`]: Loads configuration from a file, detecting `json` or `toml` files based on the file extension. Requires the `json` and `toml` feature respectively. (`toml` is enabled by default.)
- [`TomlSource`]: Loads configuration from a TOML string literal. Requires the `toml` feature. (Enabled by default.)
- [`JsonSource`]: Loads configuration from a JSON string literal. Requires the `json` feature.

## Secrets

Fields annotated with `#[confik(secret)]` will only be read from secure sources. This serves as a runtime check that no secrets have been stored in insecure places such as world-readable files.

If a secret is found in an insecure source, an error will be returned. You can opt into loading secrets on a source-by-source basis.

## Foreign Types

This crate provides implementations of [`Configuration`] for a number of `std` types and the following third-party crates. Implementations for third-party crates are feature gated.

- `chrono`: v0.4
- `rust_decimal`: v1
- `url`: v1
- `uuid`: v1
- `secrecy`: v0.8

## Macro usage

The derive macro is called `Configuration` and is used as normal:

```
#[derive(confik::Configuration)]
struct Config {
    data: usize,
}
```

### Forwarding Attributes To `Deserialize`

The serde attributes used for customizing a `Deserialize` derive typically are achieved by adding `#[confik(forward_serde(...))` attributes.

For example:

```
#[derive(confik::Configuration)]
struct Config {
    #[confik(forward_serde(rename = "other_data"))]
    data: usize,
}
```

### Defaults

Defaults are specified on a per-field basis.

- Defaults only apply if no data has been read for that field. E.g., if `data` in the below example has one value read in, it will return an error.

  ```
  # #[cfg(feature = "toml")]
  # {
  use confik::{Configuration, TomlSource};

  #[derive(Debug, Configuration)]
  struct Data {
      a: usize,
      b: usize,
  }

  #[derive(Debug, Configuration)]
  struct Config {
      #[confik(default = Data  { a: 1, b: 2 })]
      data: Data
  }

  // Data is not specified, the default is used.
  let config = Config::builder()
      .try_build()
      .unwrap();
  assert_eq!(config.data.a, 1);

  let toml = r#"
      [data]
      a = 1234
  "#;

  // Data is partially specified, but is insufficient to create it. The default is not used
  // and an error is returned.
  let config = Config::builder()
      .override_with(TomlSource::new(toml))
      .try_build()
      .unwrap_err();

  let toml = r#"
      [data]
      a = 1234
      b = 4321
  "#;

  // Data is fully specified and the default is not used.
  let config = Config::builder()
      .override_with(TomlSource::new(toml))
      .try_build()
      .unwrap();
  assert_eq!(config.data.a, 1234);
  # }
  ```

- Defaults can be given by any rust expression, and have [`Into::into`] run over them. E.g.,

  ```
  const DEFAULT_VALUE: u8 = 4;

  #[derive(confik::Configuration)]
  struct Config {
      #[confik(default = DEFAULT_VALUE)]
      a: u32,
      #[confik(default = "hello world")]
      b: String,
      #[confik(default = 5f32)]
      c: f32,
  }
  ```

- Alternatively, a default without a given value called [`Default::default`]. E.g.,

  ```
  use confik::{Configuration};

  #[derive(Configuration)]
  struct Config {
      #[confik(default)]
      a: usize
  }

  let config = Config::builder().try_build().unwrap();
  assert_eq!(config.a, 0);
  ```

### Handling Foreign Types

If there's a foreign type used in your config, then you will not be able to implement [`Configuration`] for it. Instead any type that implements [`Into`] can be used.

```
struct ForeignType {
    data: usize,
}

#[derive(confik::Configuration)]
struct MyForeignTypeCopy {
    data: usize
}

impl From<MyForeignTypeCopy> for ForeignType {
    fn from(copy: MyForeignTypeCopy) -> Self {
        Self {
            data: copy.data,
        }
    }
}

#[derive(confik::Configuration)]
struct Config {
    #[confik(from = MyForeignTypeCopy)]
    foreign_data: ForeignType
}
```

## Macro Limitations

### Custom `Deserialize` Implementations

If you're using a custom `Deserialize` implementation, then you cannot use the `Configuration` derive macro. Instead, define the necessary config implementation manually like so:

```rust
#[derive(Debug, serde_with::DeserializeFromStr)]
enum MyEnum {
    Foo,
    Bar,
};

impl std::str::FromStr for MyEnum {
    // ...
# type Err = String;
# fn from_str(_: &str) -> Result<Self, Self::Err> { unimplemented!() }
}

impl confik::Configuration for MyEnum {
    type Builder = Option<Self>;
}
```

Note that the `Option<Self>` builder type only works for simple types. For more info, see the docs on [`Configuration`] and [`ConfigurationBuilder`].
