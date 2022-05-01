use std::fmt;

pub trait ToOwnedError: std::error::Error {
    type OwnedError: std::error::Error;

    fn own(self) -> Self::OwnedError;
}

pub mod ast {
    use super::*;

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

    impl<'t> ToOwnedError for qvnt::qasm::ast::Error<'t> {
        type OwnedError = OwnedError;

        fn own(self) -> OwnedError {
            match self {
                Self::EmptySource => OwnedError::EmptySource,
                Self::ParseError(err) => OwnedError::ParseError(err.to_string()),
            }
        }
    }
}

pub mod int {
    use super::*;

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

    impl<'t> ToOwnedError for qvnt::qasm::int::Error<'t> {
        type OwnedError = OwnedError;

        fn own(self) -> OwnedError {
            match self {
                Self::NoQReg(name) => OwnedError::NoQReg(name.to_string()),
                Self::NoCReg(name) => OwnedError::NoCReg(name.to_string()),
                Self::DupQReg(name, size) => OwnedError::DupQReg(name.to_string(), size),
                Self::DupCReg(name, size) => OwnedError::DupCReg(name.to_string(), size),
                Self::IdxOutOfRange(name, idx) => OwnedError::IdxOutOfRange(name.to_string(), idx),
                Self::UnknownGate(name) => OwnedError::UnknownGate(name.to_string()),
                Self::InvalidControlMask(ctrl, act) => OwnedError::InvalidControlMask(ctrl, act),
                Self::UnevaluatedArgument(arg, err) => {
                    OwnedError::UnevaluatedArgument(arg.to_string(), err)
                }
                Self::WrongRegNumber(name, num) => {
                    OwnedError::WrongRegNumber(name.to_string(), num)
                }
                Self::WrongArgNumber(name, num) => {
                    OwnedError::WrongArgNumber(name.to_string(), num)
                }
                Self::UnmatchedRegSize(q_num, c_num) => OwnedError::UnmatchedRegSize(q_num, c_num),
                Self::MacroError(err) => OwnedError::MacroError(err.own()),
                Self::MacroAlreadyDefined(name) => {
                    OwnedError::MacroAlreadyDefined(name.to_string())
                }
                Self::DisallowedNodeInIf(node) => {
                    OwnedError::DisallowedNodeInIf(format!("{node:?}"))
                }
            }
        }
    }
}

pub mod macros {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    pub enum OwnedError {
        DisallowedNodeInMacro(String),
        DisallowedRegister(String, usize),
        UnknownReg(String),
        UnknownArg(String),
        RecursiveMacro(String),
    }

    impl fmt::Display for OwnedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                OwnedError::DisallowedNodeInMacro(node) => {
                    write!(f, "Operation {node:?} isn't allowed in Gate definition")
                }
                OwnedError::DisallowedRegister(reg, idx) => write!(
                    f,
                    "Indexing qubits ({reg}[{idx}]) isn't allowed in Gate definition"
                ),
                OwnedError::UnknownReg(reg) => {
                    write!(f, "No such register ({reg:?}) in this scope")
                }
                OwnedError::UnknownArg(arg) => {
                    write!(f, "No such argument ({arg:?}) in this scope")
                }
                OwnedError::RecursiveMacro(name) => {
                    write!(f, "Recursive macro calls ({name:?}) is not allowed")
                }
            }
        }
    }

    impl std::error::Error for OwnedError {}

    impl<'t> ToOwnedError for qvnt::qasm::int::macros::Error<'t> {
        type OwnedError = OwnedError;

        fn own(self) -> OwnedError {
            match self {
                Self::DisallowedNodeInMacro(node) => {
                    OwnedError::DisallowedNodeInMacro(format!("{node:?}"))
                }
                Self::DisallowedRegister(reg, idx) => {
                    OwnedError::DisallowedRegister(reg.to_string(), idx)
                }
                Self::UnknownReg(reg) => OwnedError::UnknownReg(reg.to_string()),
                Self::UnknownArg(arg) => OwnedError::UnknownArg(arg),
                Self::RecursiveMacro(name) => OwnedError::RecursiveMacro(name.to_string()),
            }
        }
    }
}
