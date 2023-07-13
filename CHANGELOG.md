# Changelog

## Unreleased

## 0.8.0

- **BREAKING CHANGE**: Switch `default` and `from` attributes to take bare parameters. E.g.
	- Old style: `struct MyStruct(#[config(default = "\"Hello World\"") String)`
	- New style: `struct MyStruct(#[config(default = "Hello World") String)`
	- Old style: `struct MyStruct(#[config(default = "5usize") usize)`
	- New style: `struct MyStruct(#[config(default = 5usize) usize)`

## 0.7.0

- Initial release.
