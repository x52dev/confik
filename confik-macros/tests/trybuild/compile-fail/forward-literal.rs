#[derive(confik::Configuration)]
#[confik(forward("hello world"))]
struct A;

fn main() {}
