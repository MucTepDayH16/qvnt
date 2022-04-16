use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Error<'t> {
    EmptySource,
    ParseError(qasm::Error<'t>),
}

impl<'t> fmt::Display for Error<'t> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptySource => write!(f, "Given an empty source"),
            Error::ParseError(err) => write!(f, "Parser error: {err:?}"),
        }
    }
}

impl<'t> crate::qasm::utils::ToOwnedError for Error<'t> {
    type OwnedError = OwnedError;

    fn own(self) -> OwnedError {
        match self {
            Error::EmptySource => OwnedError::EmptySource,
            Error::ParseError(err) => OwnedError::ParseError(err.to_string()),
        }
    }
}

impl<'t> std::error::Error for Error<'t> {}

#[derive(Debug, PartialEq, Clone)]
pub enum OwnedError {
    EmptySource,
    ParseError(String),
}

impl fmt::Display for OwnedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwnedError::EmptySource => write!(f, "Given an empty source"),
            OwnedError::ParseError(err) => write!(f, "Parser error: {err:?}"),
        }
    }
}

impl std::error::Error for OwnedError {}

pub type Result<'t, T> = std::result::Result<T, Error<'t>>;
