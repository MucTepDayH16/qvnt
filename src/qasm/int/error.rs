use {qasm::AstNode, std::fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    NoQReg(String),
    NoCReg(String),
    IdxOutOfRange(String, usize),
    UnknownGate(String),
    InvalidControlMask(usize, usize),
    UnevaluatedArgument(String, meval::Error),
    WrongRegNumber(String, usize),
    WrongArgNumber(String, usize),
    UnmatchedRegSize(usize, usize),
    MacroError(super::macros::Error),
    DisallowedNodeInIf(AstNode),
}

impl From<super::macros::Error> for Error {
    fn from(err: super::macros::Error) -> Self {
        Error::MacroError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoQReg(name) =>
                write!(f, "There's no quantum register, called {name:?}. Ensure to add this code: qreg {name}[SIZE]"),
            Error::NoCReg(name) =>
                write!(f, "there's no classical register, called {name:?}. Ensure to add this code: creg {name}[*SIZE*]"),
            Error::IdxOutOfRange(name, idx) =>
                write!(f, "index (={idx}) is out of bounds for register: {name}[{idx}]"),
            Error::UnknownGate(name) =>
                write!(f, "there's no quantum gate, called {name:?}"),
            Error::InvalidControlMask(ctrl, act) =>
                write!(f, "Control mask ({ctrl}) should not overlap with operators' qubits ({act})"),
            Error::UnevaluatedArgument(arg, err) =>
                write!(f, "Cannot evaluate gate argument [{arg}]: {err:?}"),
            Error::WrongRegNumber(name, num) =>
                write!(f, "Gate {name:?} cannot receive [{num}] register(s)"),
            Error::WrongArgNumber(name, num) =>
                write!(f, "Gate {name:?} cannot receive [{num}] arguments"),
            Error::UnmatchedRegSize(q_num, c_num) =>
                write!(f, "Cannot measure [{q_num}] quantum registers into [{c_num}] classical registers"),
            Error::MacroError(err) =>
                write!(f, "{err}"),
            Error::DisallowedNodeInIf(node) =>
                write!(f, "Operation {node:?} isn't allowed in If block")
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
