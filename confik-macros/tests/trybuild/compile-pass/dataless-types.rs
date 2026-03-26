//! Check that structs with no data can be built
use confik::Configuration;

#[derive(Configuration, Debug)]
struct A;

#[derive(Configuration, Debug)]
struct B {}

#[derive(Configuration, Debug)]
struct C();

fn main() {
    let _builder = A::builder().try_build().expect("No data required");
    let _builder = B::builder().try_build().expect("No data required");
    let _builder = C::builder().try_build().expect("No data required");
}
