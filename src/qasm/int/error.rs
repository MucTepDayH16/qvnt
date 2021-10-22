#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    NoQReg(String),
    NoCReg(String),
    OutOfBounds(String, usize),
    UnknownGate(String),
    UnevaluatedArgument(String),
    WrongRegNumber(String, usize),
    UnmatchedRegSize(usize, usize)
}

pub type Result<T> = std::result::Result<T, Error>;