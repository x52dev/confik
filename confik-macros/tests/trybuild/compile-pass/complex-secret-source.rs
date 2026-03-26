use confik::{ConfigBuilder, Configuration, Error, TomlSource};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq)]
struct Num(usize);

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq)]
struct PartiallySecret {
    public: Num,
    #[confik(secret)]
    secret: Num,
}

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq)]
struct NotSecret {
    public: PartiallySecret,
}

fn main() {
    let target = ConfigBuilder::<NotSecret>::default()
        .override_with(TomlSource::new("[public]\npublic = 1\nsecret = 2"))
        .try_build()
        .expect_err("Can't build secret from Toml");

    assert_matches::assert_matches!(
            &target,
            Error::UnexpectedSecret(path, _) if path.to_string().contains("`public.secret`")
    );
}
