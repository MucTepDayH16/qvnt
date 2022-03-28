use {qasm::AstNode, std::fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    NoQReg(String),
    NoCReg(String),
    IdxOutOfRange(String, usize),
    UnknownGate(String),
    InvalidControlMask(usize, usize),
    UnevaluatedArgument(String),
    WrongRegNumber(String, usize),
    WrongArgNumber(String, usize),
    UnmatchedRegSize(usize, usize),
    MacroError(super::macros::Error),
    DisallowedNodeInMIf(AstNode),
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
                write!(f, "there's no quantum register, called \"{name}\". Ensure to add this code: qreg {name}[*SIZE*]", name=name),
            Error::NoCReg(name) =>
                write!(f, "there's no classical register, called \"{name}\". Ensure to add this code: creg {name}[*SIZE*]", name=name),
            Error::IdxOutOfRange(name, idx) =>
                write!(f, "index (={idx}) is out of bounds for register: {name}[{idx}]", name=name, idx=idx),
            Error::UnknownGate(name) =>
                write!(f, "there's no quantum gate, called \"{name}\"", name=name),
            Error::InvalidControlMask(ctrl, act) =>
                write!(f, "Control mask ({}) should not overlap with operators' qubits ({})", ctrl, act),
            Error::UnevaluatedArgument(arg) =>
                write!(f, "cannot evaluate gate argument [{arg}]", arg=arg),
            Error::WrongRegNumber(name, num) =>
                write!(f, "gate \"{name}\" cannot take [{num}] register(s)", name=name, num=num),
            Error::WrongArgNumber(name, num) =>
                write!(f, "gate \"{name}\" cannot take [{num}] arguments", name=name, num=num),
            Error::UnmatchedRegSize(q_num, c_num) =>
                write!(f, "cannot measure [{q_num}] quantum registers into [{c_num}] classical registers", q_num=q_num, c_num=c_num),
            Error::MacroError(err) =>
                write!(f, "{err:?}", err=err),
            Error::DisallowedNodeInMIf(node) =>
                write!(f, "such operation ({node:?}) isn't allowed in If block", node=node)
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
