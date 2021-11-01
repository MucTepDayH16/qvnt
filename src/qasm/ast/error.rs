use std::{
    fmt,
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptySource,
    NoSuchFile(PathBuf),
    CannotRead(PathBuf),
    ParseError(qasm::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptySource =>
                write!(f, "given an empty source"),
            Error::NoSuchFile(file) =>
                write!(f, "file \"{file:?}\" not found", file=file),
            Error::CannotRead(file) =>
                write!(f, "cannot read file \"{file:?}\"", file=file),
            Error::ParseError(err) =>
                write!(f, "parser error: {err:?}", err=err),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;