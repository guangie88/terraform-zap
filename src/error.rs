use std::fmt::{self, Display};
use std::{self, io};
use subprocess::{self, ExitStatus};
use toml;
use which;
use whiteread;

/// Exit status and stderr message grouping
#[derive(Debug)]
pub struct CommandErrorCapture {
    /// Exit status returned from the command
    pub exit_status: ExitStatus,

    /// Stderr message returned from the command
    pub err_msg: String,
}

impl Display for CommandErrorCapture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Command Error:\n- Exit Status: {:?}\n\n{}",
            self.exit_status, self.err_msg
        )
    }
}

impl CommandErrorCapture {
    /// Creates a new exit status and stderr message grouping
    pub fn new<S>(exit_status: ExitStatus, err_msg: S) -> CommandErrorCapture
    where
        S: Into<String>,
    {
        CommandErrorCapture {
            exit_status,
            err_msg: err_msg.into(),
        }
    }
}

/// Application error type
#[derive(Fail, From, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Command(#[cause] subprocess::PopenError),

    #[fail(display = "{}", _0)]
    CommandError(CommandErrorCapture),

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
