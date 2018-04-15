#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![deny(missing_debug_implementations, missing_docs, warnings)]

//! # terraform-zap-ignore-lib
//!
//! Contain all ignore related implementation

extern crate serde;
#[macro_use]
extern crate serde_derive;

/// Root structure to hold the ignore method type
/// Using #[serde(untagged)] at the moment due to issue
/// <https://github.com/alexcrichton/toml-rs/issues/225>
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Ignore {
    /// Variant to ignore by exact string match
    Exact {
        /// Array of full resource names to ignore
        exact: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    extern crate toml;

    use super::*;

    #[test]
    fn test_grep_invert_valid_1() {
        const CONTENT: &str = r#"
            exact = []
        "#;

        let _: Ignore = toml::from_str(CONTENT).unwrap();
    }

    #[test]
    fn test_grep_invert_valid_2() {
        const CONTENT: &str = r#"
            exact = [
                "hello",
                "world",
            ]
        "#;

        let _: Ignore = toml::from_str(CONTENT).unwrap();
    }

    #[test]
    fn test_invalid_1() {
        const CONTENT: &str = "";
        let parsed: Result<Ignore, _> = toml::from_str(CONTENT);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_2() {
        const CONTENT: &str = "[]";
        let parsed: Result<Ignore, _> = toml::from_str(CONTENT);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_3() {
        const CONTENT: &str = r#"["hello", "world"]"#;
        let parsed: Result<Ignore, _> = toml::from_str(CONTENT);
        assert!(parsed.is_err());
    }
}
