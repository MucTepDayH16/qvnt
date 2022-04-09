use {super::macros, qasm::AstNode, std::fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    NoQReg(String),
    NoCReg(String),
    DupQReg(String, usize),
    DupCReg(String, usize),
    IdxOutOfRange(String, usize),
    UnknownGate(String),
    InvalidControlMask(usize, usize),
    UnevaluatedArgument(String, meval::Error),
    WrongRegNumber(String, usize),
    WrongArgNumber(String, usize),
    UnmatchedRegSize(usize, usize),
    MacroError(macros::Error),
    MacroAlreadyDefined(String),
    DisallowedNodeInIf(AstNode),
}

impl From<macros::Error> for Error {
    fn from(err: macros::Error) -> Self {
        Error::MacroError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoQReg(name) =>
                write!(f, "There's no quantum register, called {name:?}. Ensure to add this code: qreg {name}[SIZE]"),
            Error::NoCReg(name) =>
                write!(f, "There's no classical register, called {name:?}. Ensure to add this code: creg {name}[*SIZE*]"),
            Error::DupQReg(name, size) =>
                write!(f, "Quantum register with a similar name {name:?}  already defined with \"qreg {name}[{size}]\""),
            Error::DupCReg(name, size) =>
                write!(f, "Classical register with a similar name {name:?}  already defined with \"creg {name}[{size}]\""),
            Error::IdxOutOfRange(name, idx) =>
                write!(f, "Index (={idx}) is out of bounds for register: {name}[{idx}]"),
            Error::UnknownGate(name) =>
                write!(f, "There's no quantum gate, called {name:?}"),
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
            Error::MacroAlreadyDefined(name) =>
                write!(f, "Macro with name {name:?} already defined"),
            Error::DisallowedNodeInIf(node) =>
                write!(f, "Operation {node:?} isn't allowed in If block")
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
