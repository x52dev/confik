# Changelog

## Unreleased

- Attributes that receive expressions (`default` and `from`) now need to be unquoted. E.g.:

  ```diff
  - struct MyStruct(#[config(default = "\"Hello World\"") String)
  + struct MyStruct(#[config(default = "Hello World") String)

  - struct MyStruct(#[config(default = "5usize") usize)
  + struct MyStruct(#[config(default = 5usize) usize)
  ```

## 0.7.0

- Initial release.
