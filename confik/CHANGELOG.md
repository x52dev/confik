# Changelog

## Unreleased

## 0.14.0

- Implement `Configuration` for atomic numeric and bool types.
- Implement `Configuration` for [`js_option::JsOption`](https://docs.rs/js_option/0.1.1/js_option/enum.JsOption.html)
- Add a new `confik(forward(...))` attribute. As well as allowing for forwarding general attributes to the builder, this:
  - Replaces `confik(forward_serde(...))`. E.g.
    ```rust
    #[derive(Configuration)]
    struct Config {
      #[confik(forward(serde(default)))]
      num: usize,
    }
    ```
  - Replaces `confik(derive(...))`. E.g.
    ```rust
    #[derive(Configuration)]
    #[confik(forward(derive(Hash)))]
    struct Config(usize);
    ```
- Add a new `confik(name = ...)` attribute, that provides a custom name for the `Configuration::Builder` `struct` or `enum`.
  - This will also place the builder in the local module, so that its name is in a known location
  ```rust
  #[derive(Configuration)]
  #[confik(name = Builder)]
  struct Config {}
  ```

## 0.13.0

- Update `bytesize` dependency to `2`.
- Update `ipnetwork` dependency to `0.21`.
- Minimum supported Rust version (MSRV) is now 1.70.

## 0.12.0

- Update `secrecy` dependency to `0.10`.

## 0.11.8

- Implement `Configuration` for [`chrono::NaiveDateTime`](https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDateTime.html)

## 0.11.7

- Implement `Configuration` for [`bigdecimal::BigDecimal`](https://docs.rs/bigdecimal/0.4/bigdecimal/struct.BigDecimal.html).

## 0.11.6

- Implement `Configuration` for [`bytesize::ByteSize`](https://docs.rs/bytesize/1/bytesize/struct.ByteSize.html).

## 0.11.5

- Implement `Configuration` for [`chrono::NaiveDate`](https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveDate.html).
- Implement `Configuration` for [`chrono::NaiveTime`](https://docs.rs/chrono/0.4/chrono/naive/struct.NaiveTime.html).

## 0.11.4

- Override the following lints in macro generated code: `missing_copy_implementations`, `missing_debug_implementations`, `variant_size_differences`

## 0.11.3

- Implement `Configuration` for [`camino::Utf8PathBuf`](https://docs.rs/camino/1/camino/struct.Utf8PathBuf.html).

## 0.11.2

- Implement `Configuration` for [`ipnetwork::IpNetwork`](https://docs.rs/ipnetwork/0.20/ipnetwork/enum.IpNetwork.html).

## 0.11.1

- Parsing of database kind is now case-insensitive.
- Minimum supported Rust version (MSRV) is now 1.67 due to `toml_edit` dependency.

## 0.11.0

- Add support for `#[confik(try_from = "<ty>")]` field attribute, following the rules of `from` but using `TryFrom`. This will not break existing code unless it contains manual implementations of `Configuration`.
- Add `FailedTryInto` type.
- Add `Error::TryInto` variant.
- `.try_build()` methods now use `Error` as their return type.

## 0.10.2

- Remove `Debug` implementation from configuration builders.
- Remove `Debug` requirement for leaf configuration.
- Fix `Configuration` derive with `where` clauses.

## 0.10.1

- Implement `Configuration` for [`secrecy::SecretString`](https://docs.rs/secrecy/0.8/secrecy/type.SecretString.html). This type is always considered a secret, and can only be loaded from `Source`s which `.allow_secrets()`.
- Add `SecretOption`, an alternative to `Option` as a `Configuration::Builder` for **types** which are always secret.

## 0.10.0

- The index of an unexpected secret in now included when one is found in an unkeyed container (such as a `Vec`). Note that this will provide little to no information for unsorted/arbitrarily containers like a `HashSet`.
- `Option`s will now obey `default`s when they have received no explicit configuration. Note that explicit `null`s, such as in JSON, will set the `Option` to `None`, not to the default.
- Containers, such as `Vec` and `HashMap`, will now obey `default`s when they have received no explicit configuration.
- Partial configuration will return an error, even if there is a default set at a higher level. E.g.,

  ```rust
  struct Data {
    a: usize,
    b: usize
  }

  struct Config {
    #[confik(default = Data { a: 1, b: 2 })]
    data: Data,
  }
  ```

  with configuration:

  ```toml
  [data]
  a = "5"
  ```

  will return an error indicating `b` is missing, instead of ignoring the provided configuration.

## 0.9.0

- Optional crate features no longer have the `with-` prefix, e.g.: `with-uuid` -> `uuid`.

## 0.8.0

- Attributes that receive expressions (`default` and `from`) now need to be unquoted, e.g.:

  ```diff
  - struct Config { #[config(default = "\"Hello World\"") param: String }
  + struct Config { #[config(default = "Hello World") param: String }

  - struct Config { #[config(default = "5_usize") param: usize }
  + struct Config { #[config(default = 5_usize) param: usize }
  ```

## 0.7.0

- Initial release.
