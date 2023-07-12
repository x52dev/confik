use syn::parse_str;

use super::*;

#[test]
fn secret_attribute_parsing() {
    let input = r#"
    #[derive(Configuration)]
    struct Config {
        #[confik(secret)]
        field: String,
    }
    "#;

    let parsed = parse_str(input).expect("Failed to parse input as rust code");
    let implementer = RootImplementer::from_derive_input(&parsed)
        .expect("Failed to read derive input into `RootImplementer`");
    assert!(
        implementer
            .data
            .as_ref()
            .take_struct()
            .expect("Didn't parse as struct")
            .fields[0]
            .secret
            .is_present(),
        "Failed to read secret, state: {implementer:?}"
    );
}
