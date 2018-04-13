#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![deny(missing_debug_implementations, missing_docs, warnings)]

//! # terraform-zap-ignore-lib
//!
//! Contain all ignore related implementation

extern crate serde;
#[macro_use]
extern crate serde_derive;

/// Root structure to hold the ignore method type
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Ignore {
    /// Ignore by grep -v method
    GrepInvert(Vec<String>),
}

#[cfg(test)]
mod tests {
    extern crate toml;

    use super::*;

    #[test]
    fn test_grep_invert_1() {
        let content = r#"
            grep_invert = [
                hello,
                world,
            ]
        "#;

        let _: Ignore = toml::from_str(content).unwrap();
    }
}
