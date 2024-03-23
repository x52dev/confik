use std::str::FromStr;

use bigdecimal::BigDecimal;
use confik::{Configuration, TomlSource};
use indoc::formatdoc;

#[derive(Configuration, Debug)]
struct Config {
    big_decimal: BigDecimal,
}

fn main() {
    let big_decimal = "1.414213562373095048801688724209698078569671875376948073176679737990732478462107038850387534327641573";
    let toml = formatdoc! {r#"
        big_decimal = "{big_decimal}"
    "#};

    let config = Config::builder()
        .override_with(TomlSource::new(toml))
        .try_build()
        .expect("Failed to parse config");

    assert_eq!(
        config.big_decimal,
        BigDecimal::from_str(big_decimal).unwrap()
    );
}
