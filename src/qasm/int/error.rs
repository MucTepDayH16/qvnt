use std::fmt;

use qasm::AstNode;

use super::macros;

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
    IdentIsTooLarge(&'t str, usize),
    RegisterIsTooLarge(&'t str, usize),
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
                write!(f, "Operation {node:?} isn't allowed in If block"),
            Error::IdentIsTooLarge(name, bytes_len) =>
                write!(f, "Ident {name:?} has size({bytes_len} bytes) more than 32 bytes"),
            Error::RegisterIsTooLarge(name, q_num) =>
                write!(f, "Register {name:?} hase {q_num} qubits/bits which is more than simulator is capable of to simulate"),
        }
    }
}

impl<'t> std::error::Error for Error<'t> {}

pub type Result<'t, T> = std::result::Result<T, Error<'t>>;
