use std::{collections::HashMap, ops::Deref};

use {
    crate::{
        math::{C, N, R},
        operator::MultiOp,
        qasm::int::{gates, parse},
    },
    qasm::{Argument, AstNode},
    std::{collections::BTreeMap, fmt},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    DisallowedNodeInMacro(AstNode),
    DisallowedRegister(String, N),
    UnknownReg(String),
    UnknownArg(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DisallowedNodeInMacro(node) => {
                write!(f, "Operation {node:?} isn't allowed in Gate definition")
            }
            Error::DisallowedRegister(reg, idx) => write!(
                f,
                "Indexing qubits ({reg}[{idx}]) isn't allowed in Gate definition"
            ),
            Error::UnknownReg(reg) => {
                write!(f, "No such register ({reg:?}) in this scope")
            }
            Error::UnknownArg(arg) => {
                write!(f, "No such argument ({arg:?}) in this scope")
            }
        }
    }
}

impl std::error::Error for Error {}

pub(crate) type Result<T> = std::result::Result<T, Error>;

fn argument_name(reg: &Argument) -> String {
    match reg {
        Argument::Qubit(name, _) | Argument::Register(name) => name.clone(),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Macro {
    regs: Vec<String>,
    args: Vec<String>,
    nodes: Vec<(String, Vec<Argument>, Vec<String>)>,
}

impl Macro {
    pub(crate) fn new(
        regs: Vec<String>,
        args: Vec<String>,
        nodes: Vec<AstNode>,
    ) -> super::Result<Self> {
        let nodes = nodes
            .into_iter()
            .map(|node| match node {
                AstNode::ApplyGate(name, regs_a, args_a) => {
                    for reg_a in &regs_a {
                        match reg_a {
                            Argument::Qubit(name, idx) => {
                                return Err(
                                    Error::DisallowedRegister(name.clone(), *idx as N).into()
                                )
                            }
                            Argument::Register(name) if !regs.contains(name) => {
                                return Err(Error::UnknownReg(name.clone()).into())
                            }
                            _ => continue,
                        };
                    }

                    for arg_a in &args_a {
                        match super::parse::eval_extended(arg_a.clone(), None) {
                            Err(parse::Error::UnknownVariable(arg)) if !args.contains(&arg) => {
                                return Err(Error::UnknownArg(arg).into())
                            }
                            Err(err @ (parse::Error::Function(_, _) | parse::Error::ParseError(_) | parse::Error::RPNError(_))) => {
                                return Err(super::Error::UnevaluatedArgument(arg_a.clone(), err))
                            }
                            _ => continue,
                        };
                    }

                    Ok((name, regs_a, args_a))
                }
                disallowed_node => Err(Error::DisallowedNodeInMacro(disallowed_node).into()),
            })
            .collect::<super::Result<Vec<_>>>()?;

        Ok(Self { regs, args, nodes })
    }

    pub(crate) fn process(
        &self,
        name: String,
        regs: Vec<N>,
        args: Vec<R>,
    ) -> super::Result<MultiOp> {
        if regs.len() != self.regs.len() {
            return Err(super::Error::WrongRegNumber(name, regs.len()));
        }
        if args.len() != self.args.len() {
            return Err(super::Error::WrongArgNumber(name, args.len()));
        }

        let regs: HashMap<String, N> = self.regs.iter().cloned().zip(regs).collect();
        let args: Vec<(String, R)> = self.args.iter().cloned().zip(args).collect();

        self.nodes
            .iter()
            .try_fold(MultiOp::default(), |op, (name, regs_i, args_i)| {
                let regs_i = regs_i
                    .iter()
                    .map(|reg_i| regs[&argument_name(reg_i)])
                    .collect::<Vec<_>>();

                let args_i = args_i
                    .iter()
                    .cloned()
                    .map(|arg_i| parse::eval_extended(arg_i, &args))
                    .collect::<parse::Result<Vec<_>>>()
                    .map_err(|e| super::Error::UnevaluatedArgument(name.clone(), e))?;

                Ok(op * gates::process(name.clone(), regs_i, args_i)?)
            })
    }
}
