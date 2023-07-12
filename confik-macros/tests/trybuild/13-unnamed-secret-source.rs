use confik::{ConfigBuilder, Configuration, Error, TomlSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq, Serialize)]
struct Num(usize);

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq, Serialize)]
struct PartiallySecret(Num, #[confik(secret)] Num);

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq, Serialize)]
struct NotSecret {
    public: PartiallySecret,
}

fn main() {
    assert_eq!(
        toml::to_string(&NotSecret {
            public: PartiallySecret(Num(1), Num(2)),
        })
        .expect("Sanity check serialisation"),
        "public = [1, 2]\n"
    );

    let target = ConfigBuilder::<NotSecret>::default()
        .override_with(TomlSource::new("public = [10, 20]\n"))
        .try_build()
        .expect_err("Forbid building secret from Toml");

    assert_matches::assert_matches!(
        &target,
        Error::UnexpectedSecret(path, _) if path.to_string().contains("`public.1`")
    );
}
