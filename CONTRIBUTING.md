# Contibuting to `config`

Pointers on how to contribute, aimed at simple features. Before starting, it is recommended to install [cargo-hack], due to the number of feature combinations in this crate.

## Running tests

See [test.yml](./.github/workflows/test.yml).

## New 3rd party crate support

### Terminal/Leaf items

In the likely event that the type you want to support is an end type (i.e. we don't need to handle the types internal to it), then this will be similar to most existing [third_party] items. Either the type is fully parsed or it is not. In this case, the builder can be a simple `Option<T>`, with an implementation that looks like:

```rust
use t::T;

use crate::Configuration;

impl Configuration for T {
    type Builder = Option<Self>;
}
```

### Container types

I recommend basing this off of the containers in [std_impls]. A simple example for each can be seen in `BTreeSet` and `BTreeMap`, depending on whether the container is indexed with a key on deserialization.

[cargo-hack]: https://github.com/taiki-e/cargo-hack
[third_party]: ./src/third_party.rs
[std_impls]: ./src/std_impls.rs
