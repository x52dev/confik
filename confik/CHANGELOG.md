# Changelog

## Unreleased

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
