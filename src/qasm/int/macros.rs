use std::{collections::HashMap, fmt};

use qasm::{Argument, AstNode};

use crate::{
    math::{C, N, R},
    operator::MultiOp,
    qasm::int::{gates, parse},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Error<'t> {
    DisallowedNodeInMacro(AstNode<'t>),
    DisallowedRegister(&'t str, N),
    UnknownReg(&'t str),
    UnknownArg(String),
    RecursiveMacro(&'t str),
}

impl<'t> fmt::Display for Error<'t> {
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
            Error::RecursiveMacro(name) => {
                write!(f, "Recursive macro calls ({name:?}) is not allowed")
            }
        }
    }
}

impl<'t> std::error::Error for Error<'t> {}

pub(crate) type Result<'t, T> = std::result::Result<T, Error<'t>>;

fn argument_name<'t>(reg: Argument<'t>) -> &'t str {
    match reg {
        Argument::Qubit(name, _) | Argument::Register(name) => name,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Macro<'t> {
    regs: Vec<&'t str>,
    args: Vec<&'t str>,
    nodes: Vec<(&'t str, Vec<Argument<'t>>, Vec<&'t str>)>,
}

impl<'t> Macro<'t> {
    pub(crate) fn new(
        regs: Vec<&'t str>,
        args: Vec<&'t str>,
        nodes: Vec<AstNode<'t>>,
    ) -> super::Result<'t, Self> {
        let nodes = nodes
            .into_iter()
            .map(|node| match node {
                AstNode::ApplyGate(name, regs_a, args_a) => {
                    for reg_a in &regs_a {
                        match reg_a.clone() {
                            Argument::Qubit(name, idx) => {
                                return Err(Error::DisallowedRegister(
                                    name, idx as N,
                                )
                                .into())
                            }
                            Argument::Register(name)
                                if !regs.contains(&name) =>
                            {
                                return Err(Error::UnknownReg(name).into())
                            }
                            _ => continue,
                        };
                    }

                    for arg_a in &args_a {
                        match super::parse::eval_extended(arg_a, None) {
                            Err(parse::Error::UnknownVariable(arg))
                                if !args.contains(&&*arg) =>
                            {
                                return Err(Error::UnknownArg(arg).into())
                            }
                            Err(
                                err @ (parse::Error::Function(_, _)
                                | parse::Error::ParseError(_)
                                | parse::Error::RPNError(_)),
                            ) => {
                                return Err(super::Error::UnevaluatedArgument(
                                    arg_a,
                                    err,
                                ))
                            }
                            _ => continue,
                        };
                    }

                    Ok((name, regs_a, args_a))
                }
                disallowed_node => {
                    Err(Error::DisallowedNodeInMacro(disallowed_node).into())
                }
            })
            .collect::<super::Result<Vec<_>>>()?;

        Ok(Self { regs, args, nodes })
    }

    pub(crate) fn process(
        &self,
        name: &'t str,
        regs: Vec<N>,
        args: Vec<R>,
        macros: &HashMap<&'t str, Macro<'t>>,
    ) -> super::Result<'t, MultiOp> {
        if regs.len() != self.regs.len() {
            return Err(super::Error::WrongRegNumber(name, regs.len()));
        }
        if args.len() != self.args.len() {
            return Err(super::Error::WrongArgNumber(name, args.len()));
        }

        let regs: HashMap<&'t str, N> =
            self.regs.iter().cloned().zip(regs).collect();
        let args: Vec<(&'t str, R)> =
            self.args.iter().cloned().zip(args).collect();

        self.nodes.iter().try_fold(
            MultiOp::default(),
            |op, (name_i, regs_i, args_i)| {
                let regs_i = regs_i
                    .iter()
                    .cloned()
                    .map(|reg_i| regs[&argument_name(reg_i)])
                    .collect::<Vec<_>>();

                let args_i = args_i
                    .iter()
                    .cloned()
                    .map(|arg_i| parse::eval_extended(arg_i, args.clone()))
                    .collect::<parse::Result<Vec<_>>>()
                    .map_err(|e| {
                        super::Error::UnevaluatedArgument(name_i, e)
                    })?;

                let op_res = match macros.get(*name_i) {
                    Some(_macro) => {
                        if &name == name_i {
                            return Err(Error::RecursiveMacro(name_i).into());
                        }
                        _macro.process(name_i, regs_i, args_i, macros)?
                    }
                    None => gates::process(name_i, regs_i, args_i)?,
                };
                Ok(op * op_res)
            },
        )
    }
}

#[cfg(test)]
pub(crate) fn dummy_macro() -> Macro<'static> {
    Macro {
        regs: vec!["a", "b"],
        args: vec!["x", "y"],
        nodes: vec![
            ("h", vec![Argument::Register("x")], vec![]),
            ("x", vec![Argument::Register("x")], vec![]),
        ],
    }
}
