use confik::Configuration;

#[allow(dead_code)] // unused in no-default-features cases
#[derive(Debug, Configuration, Eq, PartialEq)]
struct NonCopy(usize);

#[allow(dead_code)] // unused in no-default-features cases
#[derive(Debug, Configuration, Eq, PartialEq)]
struct Target {
    val: [NonCopy; 10],
}

#[cfg(feature = "toml")]
mod toml {
    use confik::{Configuration, TomlSource};

    use super::{NonCopy, Target};

    #[test]
    fn success() {
        let target = Target::builder()
            .override_with(TomlSource::new("val = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"))
            .try_build()
            .expect("Failed to build array");
        assert_eq!(
            target,
            Target {
                val: [
                    NonCopy(0),
                    NonCopy(1),
                    NonCopy(2),
                    NonCopy(3),
                    NonCopy(4),
                    NonCopy(5),
                    NonCopy(6),
                    NonCopy(7),
                    NonCopy(8),
                    NonCopy(9)
                ]
            }
        );
    }
}

#[cfg(feature = "json")]
mod json {
    use assert_matches::assert_matches;
    use confik::{Configuration, Error, JsonSource};

    use super::{NonCopy, Target};

    #[test]
    fn too_short() {
        Target::builder()
            .override_with(JsonSource::new("{\"val\": [0, 1, 2, 3, 4, 5, 6, 7, 8]}"))
            .try_build()
            .expect_err("Built array too short");
    }

    #[test]
    fn too_short_with_null() {
        let err = Target::builder()
            .override_with(JsonSource::new(
                "{\"val\": [0, 1, 2, 3, 4, 5, 6, 7, 8, null]}",
            ))
            .try_build()
            .expect_err("Built array too short");
        assert_matches!(err, Error::MissingValue(path) if path.to_string().contains("val.9.0"));
    }

    #[test]
    fn too_long() {
        Target::builder()
            .override_with(JsonSource::new(
                "{\"val\": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]}",
            ))
            .try_build()
            .expect_err("Built array too long");
    }

    #[test]
    fn merge() {
        let target = Target::builder()
            .override_with(JsonSource::new(
                "{\"val\": [0, null, 2, null, 4, null, 6, null, 8, null]}",
            ))
            .override_with(JsonSource::new(
                "{\"val\": [null, 1, null, 3, null, 5, null, 7, null, 9]}",
            ))
            .try_build()
            .expect("Merged array failure");
        assert_eq!(
            target,
            Target {
                val: [
                    NonCopy(0),
                    NonCopy(1),
                    NonCopy(2),
                    NonCopy(3),
                    NonCopy(4),
                    NonCopy(5),
                    NonCopy(6),
                    NonCopy(7),
                    NonCopy(8),
                    NonCopy(9)
                ]
            }
        );
    }
}
