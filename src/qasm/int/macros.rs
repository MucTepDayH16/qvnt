use {
    std::collections::BTreeMap,
    qasm::{Argument, AstNode},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    DisallowedNodeInMacro(AstNode),
    UnmatchedRegInput(usize, usize),
    UnmatchedArgInput(usize, usize),
    UnknownReg(Argument),
    UnknownArg(String),
}

#[derive(Debug)]
pub (crate) struct ProcessMacro {
    regs: BTreeMap<String, usize>,
    args: BTreeMap<String, usize>,
    nodes: Vec<AstNode>,
}

pub (crate) type Result<T> = std::result::Result<T, Error>;

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
                    Some(reg) => return Err(Error::UnknownReg(reg.clone())),
                    None => {},
                };

                match a_args.iter().find(
                    |arg| super::parse_arg(arg).is_none() && !args.contains(arg)) {
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

    pub (crate) fn apply(&self, regs: &Vec<Argument>, args: &Vec<String>) -> Result<Vec<AstNode>> {
        if regs.len() != self.regs.len() {
            return Err(Error::UnmatchedRegInput(regs.len(), self.regs.len()));
        }
        if args.len() != self.args.len() {
            return Err(Error::UnmatchedArgInput(args.len(), self.args.len()));
        }

        self.nodes.iter().try_fold(
            Vec::<AstNode>::new(),
            |mut vec, node| {
                let node = if let AstNode::ApplyGate(name, regs1, args1) = node {
                    let regs1 = regs1.iter()
                        .map(|reg| {
                            let idx = self.regs[&argument_name(reg)];
                            regs[idx].clone()
                        }).collect();

                    let args1 = args1.iter()
                        .map(|arg| {
                            if super::parse_arg(arg).is_some() {
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
                };

                vec.push(node);
                Ok(vec)
            }
        )
    }
}