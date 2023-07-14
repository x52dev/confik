# Changelog

## Unreleased

## 0.10.0

- Include the index of an unexpected secret when one is found in an unkeyed container (such as a `Vec`). Note that this will provide little to no information for unsorted/arbitrarily containers like a `HashSet`.
- `Option`s will now obey `default`s when they have received no explicit configuration.
  - Explicit `null`s, such as in JSON will set the `Option` to `None`, not to the default.
- Containers, such as `Vec` and `HashMap` will now obey `default`s when they have received no explicit configuration.
  - Explicit empty sets will create an empty container.
- Partial configuration will return an error, even if there is a default set at a higher level. E.g.
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
