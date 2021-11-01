use {
    std::{
        collections::BTreeMap,
        fmt,
    },
    qasm::{Argument, AstNode},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    DisallowedNodeInMacro(AstNode),
    UnknownReg(String),
    UnknownArg(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DisallowedNodeInMacro(node) =>
                write!(f, "such operation ({node:?}) isn't allowed in Gate definition", node=node),
            Error::UnknownReg(reg) =>
                write!(f, "no such register ({reg:?}) in this scope", reg=reg),
            Error::UnknownArg(arg) =>
                write!(f, "no such argument ({arg:?}) in this scope", arg=arg),
        }
    }
}

impl std::error::Error for Error {}

pub (crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub (crate) struct ProcessMacro {
    regs: BTreeMap<String, usize>,
    args: BTreeMap<String, usize>,
    nodes: Vec<AstNode>,
}

fn argument_name(reg: &Argument) -> String {
    match reg {
        Argument::Qubit(name, _) => name.clone(),
        Argument::Register(name) => name.clone(),
    }
}

impl ProcessMacro {
    pub (crate) fn new(regs: Vec<String>, args: Vec<String>, nodes: Vec<AstNode>) -> Result<Self> {
        nodes.iter().try_for_each(|node| match node {
            AstNode::ApplyGate(_, a_regs, a_args) => {
                match a_regs.iter().find(
                    |reg| !regs.contains(&argument_name(reg))) {
                    Some(reg) => return Err(Error::UnknownReg(argument_name(reg))),
                    None => {},
                };

                match a_args.iter().find(
                    |arg| super::parse::eval(arg).is_none() && !args.contains(arg)) {
                    Some(arg) => return Err(Error::UnknownArg(arg.clone())),
                    None => {},
                }

                Ok(())
            },
            x => Err(Error::DisallowedNodeInMacro(x.clone())),
        })?;

        let regs = regs.iter()
            .enumerate()
            .map(|(idx, reg)| (reg.clone(), idx))
            .collect();

        let args = args.iter()
            .enumerate()
            .map(|(idx, arg)| (arg.clone(), idx))
            .collect();

        Ok(Self{ regs, args, nodes, })
    }

    pub (crate) fn apply(&self, name: &String, regs: &Vec<Argument>, args: &Vec<String>) -> super::Result<Vec<AstNode>> {
        if regs.len() != self.regs.len() {
            return Err(super::Error::WrongRegNumber(name.clone(), regs.len()));
        }
        if args.len() != self.args.len() {
            return Err(super::Error::WrongArgNumber(name.clone(), args.len()));
        }

        let nodes = self.nodes.iter().map(
            |node| {
                if let AstNode::ApplyGate(name, regs1, args1) = node {
                    let regs1 = regs1.iter()
                        .map(|reg| {
                            let idx = self.regs[&argument_name(reg)];
                            regs[idx].clone()
                        }).collect();

                    let args1 = args1.iter()
                        .map(|arg| {
                            if let Some(_) = super::parse::eval(arg) {
                                arg.clone()
                            } else {
                                let idx = self.args[arg];
                                args[idx].clone()
                            }
                        }).collect();

                    AstNode::ApplyGate(
                        name.clone(),
                        regs1,
                        args1,
                    )
                } else {
                    unreachable!()
                }
            }
        ).collect();
        Ok(nodes)
    }
}