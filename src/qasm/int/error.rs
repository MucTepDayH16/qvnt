use {super::macros, qasm::AstNode, std::fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum Error<'t> {
    NoQReg(&'t str),
    NoCReg(&'t str),
    DupQReg(&'t str, usize),
    DupCReg(&'t str, usize),
    IdxOutOfRange(&'t str, usize),
    UnknownGate(&'t str),
    InvalidControlMask(usize, usize),
    UnevaluatedArgument(&'t str, meval::Error),
    WrongRegNumber(&'t str, usize),
    WrongArgNumber(&'t str, usize),
    UnmatchedRegSize(usize, usize),
    MacroError(macros::Error<'t>),
    MacroAlreadyDefined(&'t str),
    DisallowedNodeInIf(AstNode<'t>),
}

impl<'t> From<macros::Error<'t>> for Error<'t> {
    fn from(err: macros::Error<'t>) -> Self {
        Error::MacroError(err)
    }
}

impl<'t> fmt::Display for Error<'t> {
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

impl<'t> crate::qasm::utils::ToOwnedError for Error<'t> {
    type OwnedError = OwnedError;

    fn own(self) -> OwnedError {
        match self {
            Error::NoQReg(name) => OwnedError::NoQReg(name.to_string()),
            Error::NoCReg(name) => OwnedError::NoCReg(name.to_string()),
            Error::DupQReg(name, size) => OwnedError::DupQReg(name.to_string(), size),
            Error::DupCReg(name, size) => OwnedError::DupCReg(name.to_string(), size),
            Error::IdxOutOfRange(name, idx) => OwnedError::IdxOutOfRange(name.to_string(), idx),
            Error::UnknownGate(name) => OwnedError::UnknownGate(name.to_string()),
            Error::InvalidControlMask(ctrl, act) => OwnedError::InvalidControlMask(ctrl, act),
            Error::UnevaluatedArgument(arg, err) => OwnedError::UnevaluatedArgument(arg.to_string(), err),
            Error::WrongRegNumber(name, num) => OwnedError::WrongRegNumber(name.to_string(), num),
            Error::WrongArgNumber(name, num) => OwnedError::WrongArgNumber(name.to_string(), num),
            Error::UnmatchedRegSize(q_num, c_num) => OwnedError::UnmatchedRegSize(q_num, c_num),
            Error::MacroError(err) => OwnedError::MacroError(err.own()),
            Error::MacroAlreadyDefined(name) => OwnedError::MacroAlreadyDefined(name.to_string()),
            Error::DisallowedNodeInIf(node) => OwnedError::DisallowedNodeInIf(format!("{node:?}")),
        }
    }
}

impl<'t> std::error::Error for Error<'t> {}

#[derive(Debug, PartialEq, Clone)]
pub enum OwnedError {
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
    MacroError(macros::OwnedError),
    MacroAlreadyDefined(String),
    DisallowedNodeInIf(String),
}

impl fmt::Display for OwnedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwnedError::NoQReg(name) =>
                write!(f, "There's no quantum register, called {name:?}. Ensure to add this code: qreg {name}[SIZE]"),
            OwnedError::NoCReg(name) =>
                write!(f, "There's no classical register, called {name:?}. Ensure to add this code: creg {name}[*SIZE*]"),
            OwnedError::DupQReg(name, size) =>
                write!(f, "Quantum register with a similar name {name:?}  already defined with \"qreg {name}[{size}]\""),
            OwnedError::DupCReg(name, size) =>
                write!(f, "Classical register with a similar name {name:?}  already defined with \"creg {name}[{size}]\""),
            OwnedError::IdxOutOfRange(name, idx) =>
                write!(f, "Index (={idx}) is out of bounds for register: {name}[{idx}]"),
            OwnedError::UnknownGate(name) =>
                write!(f, "There's no quantum gate, called {name:?}"),
            OwnedError::InvalidControlMask(ctrl, act) =>
                write!(f, "Control mask ({ctrl}) should not overlap with operators' qubits ({act})"),
            OwnedError::UnevaluatedArgument(arg, err) =>
                write!(f, "Cannot evaluate gate argument [{arg}]: {err:?}"),
            OwnedError::WrongRegNumber(name, num) =>
                write!(f, "Gate {name:?} cannot receive [{num}] register(s)"),
            OwnedError::WrongArgNumber(name, num) =>
                write!(f, "Gate {name:?} cannot receive [{num}] arguments"),
            OwnedError::UnmatchedRegSize(q_num, c_num) =>
                write!(f, "Cannot measure [{q_num}] quantum registers into [{c_num}] classical registers"),
            OwnedError::MacroError(err) =>
                write!(f, "{err}"),
            OwnedError::MacroAlreadyDefined(name) =>
                write!(f, "Macro with name {name:?} already defined"),
            OwnedError::DisallowedNodeInIf(node) =>
                write!(f, "Operation {node:?} isn't allowed in If block")
        }
    }
}

impl std::error::Error for OwnedError {}

pub type Result<'t, T> = std::result::Result<T, Error<'t>>;
