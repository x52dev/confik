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

## Macro usage

The derive macro is called `Configuration` and is used as normal:

```rust
#[derive(confik::Configuration)]
struct Config {
    data: usize,
}
```

### Forwarding Attributes

This allows forwarding any kind of attribute on to the builder.

#### Serde

The serde attributes used for customizing a `Deserialize` derive are achieved by adding `#[confik(forward(serde(...)))]` attributes.

For example:

```rust
# use confik::Configuration;
#[derive(Configuration, Debug, PartialEq, Eq)]
struct Field {
    #[confik(forward(serde(rename = "other_name")))]
    field1: usize,
}
```

#### Derives

If you need additional derives for your type, these can be added via `#[confik(forward(derive...))]` attributes.

For example:

```rust
# use confik::Configuration;
#[derive(Debug, Configuration, Hash, Eq, PartialEq)]
#[confik(forward(derive(Hash, Eq, PartialEq)))]
struct Value {
    inner: String,
}
```

### Defaults

Defaults are specified on a per-field basis.

- Defaults only apply if no data has been read for that field. E.g., if `data` in the below example has one value read in, it will return an error.

  ```rust
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

  ```rust
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

  ```rust
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

This crate provides implementations of [`Configuration`] for a number of `std` types and the following third-party crates. Implementations for third-party crates are feature gated.

- `ahash`: v0.8
- `bigdecimal`: v0.4
- `bytesize`: v2
- `camino`: v1
- `chrono`: v0.4
- `ipnetwork`: v0.21
- `js_option`: v0.1
- `rust_decimal`: v1
- `secrecy`: v0.10 (Note that `#[config(secret)]` is not needed, although it is harmless, for these types as they are always treated as secrets.)
- `url`: v1
- `uuid`: v1

If there's another foreign type used in your config, then you will not be able to implement [`Configuration`] for it. Instead any type that implements [`Into`] or [`TryInto`] can be used.

```rust
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
struct MyForeignTypeIsize {
    data: isize
}

impl TryFrom<MyForeignTypeIsize> for ForeignType {
    type Error = <usize as TryFrom<isize>>::Error;

    fn try_from(copy: MyForeignTypeIsize) -> Result<Self, Self::Error> {
        Ok(Self {
            data: copy.data.try_into()?,
        })
    }
}

#[derive(confik::Configuration)]
struct Config {
    #[confik(from = MyForeignTypeCopy)]
    foreign_data: ForeignType,

    #[confik(try_from = MyForeignTypeIsize)]
    foreign_data_isized: ForeignType,
}
```

### Named builders

If you want to directly access the builders, you can provide them with a name. This will also place the builder in the local module, to ensure there's a known path with which to reference them.

```rust
#[derive(confik::Configuration)]
#[confik(name = Builder)]
struct Config {
    data: usize,
}

let _ = Builder { data: Default::default() };
```

### Field and Builder visibility

Field and builder visibility are directly inherited from the underlying type. E.g.

```rust
mod config {
    #[derive(confik::Configuration)]
    pub struct Config {
        pub data: usize,
    }
}

// Required as you can't use this syntax for struct initialisation.
type Builder = <config::Config as confik::Configuration>::Builder;

let _ = Builder { data: Default::default() };
```

### Skipping fields

Fields can be skipped if necessary. This allows having types that cannot implement `Configuration` or be deserializable. However the field must have a `confik(default)` or `confik(default = ...)` attribute, otherwise it can't be built. E.g.

```rust
# use std::time::Instant;
#[derive(confik::Configuration)]
struct Config {
  #[confik(skip, default = Instant::now())]
  loaded_at: Instant,
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

## Manual implementations

It is strongly recommended to use the `derive` macro where possible. However, there may be cases where this is not possible. For some cases there are additional attributes available in the `derive` macro to tweak the behaviour, see the section on Handling Foreign Types.

If you would like to manually implement `Configuration` for a type anyway, then this can mostly be broken down to three cases.

### Simple cases

If your type cannot be partial specified (e.g. `usize`, `String`), then a simple `Option<Self>` builder can be used.

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

### Containers

Unless your container holds another container, which already implements `Configuration`, you'll likely need to implement `Configuration` yourself, instead of with a `derive`. There are two type of containers that may need to be handled here.

#### Keyed Containers

Keyed containers have their contents separate from their keys. Examples of these are [`HashMap`](std::collections::HashMap) and [`BTreeMap`](std::collections::BTreeMap). Whilst the implementations can be provided fully, there are helpers available. These are the [`KeyedContainerBuilder`][KeyedContainerBuilder] type and the [`KeyedContainer`][KeyedContainer] trait.

A type which implements all of [`KeyedContainer`][KeyedContainer], [`Deserialize`][serde::Deserialize], [`FromIterator`], [`Default`], and [`IntoIterator`] (for both the type and a reference to the type) can then use [`KeyedContainerBuilder`][KeyedContainerBuilder] as their builder. See [`KeyedContainerBuilder`][KeyedContainerBuilder] for an example.

Note that the key needs to implement `Display` so that an accurate error stack can be generated.

[KeyedContainerBuilder]: crate::helpers::KeyedContainerBuilder
[KeyedContainer]: crate::helpers::KeyedContainer

#### Unkeyed Containers

Unkeyed containers are types without a separate key. This includes [`Vec`], but also types like [`HashSet`](std::collections::HashSet). Whilst the implementations can be provided fully, there is a helper available. This is the [`UnkeyedContainerBuilder`][UnkeyedContainerBuilder].

A type which implements all of [`Deserialize`][serde::Deserialize], [`FromIterator`], [`Default`], and [`IntoIterator`] (for both the type and a reference to the type) can then use [`UnkeyedContainerBuilder`][UnkeyedContainerBuilder] as their builder. See [`UnkeyedContainerBuilder`][UnkeyedContainerBuilder] for an example.

[UnkeyedContainerBuilder]: crate::helpers::UnkeyedContainerBuilder

#### Other complex cases

For other complex cases, where `derive`s cannot work, the type is not simple enough to use an `Option<Self>` builder, and is not a container, there is currently no additional support. Please read through the [`Configuration`] and [`ConfigurationBuilder`] traits and implement them as appropriate.

If you believe your type is following a common pattern where we could provide more support, please raise an issue (or even better an MR).
