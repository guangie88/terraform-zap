use std::{self, io};
use subprocess;
use toml;
use which;
use whiteread;

/// Application error type
#[derive(Fail, From, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Command(#[cause] subprocess::PopenError),

    #[fail(display = "{}", _0)]
    CommandError(String),

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    ParseString(#[cause] whiteread::white::Error),

    #[fail(display = "Given path not an executable")]
    NotExecutable,

    #[fail(display = "{}", _0)]
    TomlDe(#[cause] toml::de::Error),

    #[fail(display = "{}", _0)]
    Which(#[cause] which::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
