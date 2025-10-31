use confik::Configuration;
use serde::de::DeserializeOwned;

trait MyTrait {
    type Config: Configuration;
}

#[derive(Configuration)]
struct EmptyConfig;

impl MyTrait for () {
    type Config = EmptyConfig;
}

#[derive(Configuration)]
#[confik(forward(serde(bound = "C: MyTrait + DeserializeOwned")))]
struct Config<C>
where
    C: MyTrait + Default + DeserializeOwned,
{
    sub_config: <C as MyTrait>::Config,
}

fn main() {
    Config::<()>::builder()
        .try_build()
        .expect("No configuration needed");
}
