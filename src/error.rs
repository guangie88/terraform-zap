use std;

/// Application error type
#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Lorem")]
    Lorem,
}

pub type Result<T> = std::result::Result<T, Error>;
