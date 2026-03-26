use confik::Configuration;

#[derive(Configuration, Debug)]
enum E {
    V {
        #[confik(struct_default)]
        x: u32,
    },
}

fn main() {}
