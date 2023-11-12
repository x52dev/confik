# Changelog

## Unreleased

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
