//! Checks "untagged enum" behavior / fall-through when deserializing.

use confik::{Configuration, TomlSource};

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum S3Auth {
    K8sServiceAccount {
        use_k8s_service_account: serde_bool::True,
    },

    ApiKey {
        access_key: String,
        access_secret: String,
    },
}

impl Configuration for S3Auth {
    type Builder = Option<Self>;
}

#[derive(Debug, Configuration)]
struct Config {
    s3_auth: S3Auth,
}

fn main() {
    let config = indoc::formatdoc! {"
        [s3_auth]
        use_k8s_service_account = false
    "};
    Config::builder()
        .override_with(TomlSource::new(config))
        .try_build()
        .expect_err("Successfully built with incorrect boolean inner value");

    let config = indoc::formatdoc! {"
        [s3_auth]
        use_k8s_service_account = true
    "};
    let config = Config::builder()
        .override_with(TomlSource::new(config))
        .try_build()
        .expect("Failed to build");
    assert_matches::assert_matches!(config.s3_auth, S3Auth::K8sServiceAccount { .. });

    let config = indoc::formatdoc! {"
        [s3_auth]
        access_key = \"foo\"
        access_secret = \"bar\"
    "};
    let config = Config::builder()
        .override_with(TomlSource::new(config))
        .try_build()
        .expect("Failed to build");
    assert_matches::assert_matches!(config.s3_auth, S3Auth::ApiKey { .. });
}

// /// Only successfully deserializes from a `true` value boolean.
// struct True;

// impl serde::Deserialize for True {}

// impl config::Configuration for True {
//     type Builder = Option<Self>;
// }
