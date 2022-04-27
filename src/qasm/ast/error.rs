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

impl<'t> std::error::Error for Error<'t> {}

pub type Result<'t, T> = std::result::Result<T, Error<'t>>;
