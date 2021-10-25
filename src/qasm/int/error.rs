use {
    std::{
        fmt,
    },
};

#[derive(PartialEq, Clone)]
pub enum Error {
    NoQReg(String),
    NoCReg(String),
    IdxOutOfRange(String, usize),
    UnknownGate(String),
    UnevaluatedArgument(String),
    WrongRegNumber(String, usize),
    WrongArgNumber(String, usize),
    UnmatchedRegSize(usize, usize)
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoQReg(name) =>
                write!(f, "There's no quantum register, called \"{name}\". Ensure to add this code: qreg {name}[*SIZE*]", name=name),
            Error::NoCReg(name) =>
                write!(f, "There's no classical register, called \"{name}\". Ensure to add this code: creg {name}[*SIZE*]", name=name),
            Error::IdxOutOfRange(name, idx) =>
                write!(f, "Index (={idx}) is out of bounds for register: {name}[{idx}]", name=name, idx=idx),
            Error::UnknownGate(name) =>
                write!(f, "There's no quantum gate, called \"{name}\"", name=name),
            Error::UnevaluatedArgument(arg) =>
                write!(f, "Cannot evaluate gate argument [{arg}]", arg=arg),
            Error::WrongRegNumber(name, num) =>
                write!(f, "Gate \"{name}\" cannot take [{num}] register(s)", name=name, num=num),
            Error::WrongArgNumber(name, num) =>
                write!(f, "Gate \"{name}\" cannot take [{num}] arguments", name=name, num=num),
            Error::UnmatchedRegSize(q_num, c_num) =>
                write!(f, "Cannot measure [{q_num}] quantum registers into [{c_num}] classical registres", q_num=q_num, c_num=c_num),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;