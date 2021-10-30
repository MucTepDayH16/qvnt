use {
    std::{
        collections::{VecDeque, BTreeMap},
    },
    qasm::{Argument, AstNode},

    crate::{
        bits_iter::BitsIter,
        math::{C, R, N},
        operator::{MultiOp, op},
        register::{QReg, CReg},
        qasm::ast::Ast,
    },
};

mod gates;
mod error;
mod macros;
mod parse;

use error::{Error, Result};
use macros::ProcessMacro;

#[derive(Clone, Debug, PartialEq)]
enum MeasureOp {
    Set, Xor
}

#[derive(Clone, Debug, PartialEq)]
pub (crate) enum Sep {
    Nop,
    Measure(usize, usize),
    IfBranch(usize, usize),
}

impl Default for MeasureOp {
    fn default() -> Self {
        MeasureOp::Set
    }
}

#[derive(Debug, Default)]
pub struct Int {
    m_op: MeasureOp,
    q_reg: (QReg, Vec<String>),
    c_reg: (CReg, Vec<String>),
    q_ops: (MultiOp, VecDeque<(MultiOp, Sep)>),
    macros: BTreeMap<String, ProcessMacro>
}

impl Int {
    pub fn new(ast: &Ast) -> Result<Self> {
        Self::default().process_nodes(ast.iter())
    }

    pub fn xor(self) -> Self {
        Self{ m_op: MeasureOp::Xor, ..self }
    }

    pub fn finish(&mut self) -> &mut Self {
        let ops = std::mem::take(&mut self.q_ops.1);
        for (op, sep) in ops.iter() {
            match sep {
                Sep::Nop => {
                    self.q_reg.0.apply(op);
                }
                Sep::Measure(q, c) => {
                    self.q_reg.0.apply(op);
                    self.measure(*q, *c);
                },
                Sep::IfBranch(c, v) => {
                    if self.c_reg.0.get_value(*c) == *v {
                        self.q_reg.0.apply(op);
                    }
                },
            }
        }
        self.q_reg.0.apply(&self.q_ops.0);
        self.q_ops.1 = ops;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.q_reg.0.reset(0);
        self.c_reg.0.reset(0);
        self
    }

    fn process_nodes<'a, I: Iterator<Item=&'a AstNode>>(self, mut nodes: I) -> Result<Self> {
        nodes.try_fold(self, |this, node| {
            this.process_node(node)
        })
    }

    fn process_node(mut self, node: &AstNode) -> Result<Self> {
        match node {
            AstNode::QReg(alias, size) => self.add_q_reg(alias, *size as N),
            AstNode::CReg(alias, size) => self.add_c_reg(alias, *size as N),
            AstNode::Barrier(_) => todo!(),
            AstNode::Reset(_) => todo!(),
            AstNode::Measure(q_arg, c_arg) => {
                let q_arg = self.get_q_idx(q_arg)?;
                let c_arg = self.get_c_idx(c_arg)?;
                if q_arg.count_ones() != c_arg.count_ones() {
                    return Err(Error::UnmatchedRegSize(
                        q_arg.count_ones() as N,
                        c_arg.count_ones() as N
                    ));
                }

                self.branch_with_id(Sep::Measure(q_arg, c_arg));
            },
            AstNode::ApplyGate(name, regs, args) => {
                if let Some(macros) = self.macros.get(name) {
                    let nodes = macros.apply(name, regs, args)?;
                    self = self.process_nodes(nodes.iter())?;
                } else {
                    let name = name.to_lowercase();

                    let regs = regs.into_iter()
                                   .map(|reg| self.get_q_idx(reg))
                                   .collect::<Vec<Result<N>>>();
                    if let Some(err) = regs.iter().find(|reg| reg.is_err()) {
                        return Err(err.clone().unwrap_err());
                    }
                    let regs = regs.into_iter()
                                   .map(|reg| reg.unwrap())
                                   .collect();

                    let args = args.into_iter()
                                   .map(|arg| (arg, parse::parse(arg)))
                                   .collect::<Vec<(&String, Option<R>)>>();
                    if let Some(err) = args.iter().find(|arg| arg.1.is_none()) {
                        return Err(Error::UnevaluatedArgument(err.0.clone()));
                    }
                    let args = args.into_iter()
                                   .map(|arg| arg.1.unwrap())
                                   .collect();

                    self.q_ops.0 *= gates::process(name, regs, args)?;
                }
            },
            AstNode::Opaque(_, _, _) => todo!(),
            AstNode::Gate(name, regs, args, nodes) => {
                let macros = macros::ProcessMacro::new(
                    regs.clone(),
                    args.clone(),
                    nodes.clone()
                )?;

                self.macros.insert(name.clone(), macros);
            },
            AstNode::If(a, b, c) => {
                let c = c.as_ref();
                match c {
                    AstNode::ApplyGate(_, _, _) => {
                        self.branch(Sep::Nop);

                        self = self.process_node(c)?;

                        let c =self.get_c_idx(&Argument::Register(a.clone()))?;
                        self.branch(Sep::IfBranch(c, *b as N));
                    },
                    _ => return Err(Error::DisallowedNodeInMIf(c.clone())),
                }
            },
        }

        Ok(self)
    }

    fn get_idx_by_alias(&self, alias: &String) -> (N, N) {
        let q_mask = self.q_reg.1.iter()
            .enumerate()
            .fold(0, |acc, (idx, name)|
                if name == alias {
                    acc | 1_usize.wrapping_shl(idx as u32)
                } else {
                    acc
                });
        let c_mask = self.c_reg.1.iter()
            .enumerate()
            .fold(0, |acc, (idx, name)|
                if name == alias {
                    acc | 1_usize.wrapping_shl(idx as u32)
                } else {
                    acc
                });

        (q_mask, c_mask)
    }

    pub (crate) fn get_q_idx(&self, arg: &Argument) -> Result<N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(alias).0;
                if mask != 0 {
                    crate::bits_iter::BitsIter::from(mask)
                        .nth(*idx as N)
                        .ok_or(Error::IdxOutOfRange(alias.clone(), *idx as N))
                } else {
                    Err(Error::NoQReg(alias.clone()))
                }
            },
            Argument::Register(alias) => {
                let mask = self.get_idx_by_alias(alias).0;
                if mask != 0 {
                    Ok(mask)
                } else {
                    Err(Error::NoQReg(alias.clone()))
                }
            },
        }
    }
    pub (crate) fn get_c_idx(&self, arg: &Argument) -> Result<N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(alias).1;
                if mask != 0 {
                    crate::bits_iter::BitsIter::from(mask)
                        .nth(*idx as N)
                        .ok_or(Error::IdxOutOfRange(alias.clone(), *idx as N))
                } else {
                    Err(Error::NoCReg(alias.clone()))
                }
            },
            Argument::Register(alias) => {
                let mask = self.get_idx_by_alias(alias).1;
                if mask != 0 {
                    Ok(mask)
                } else {
                    Err(Error::NoCReg(alias.clone()))
                }
            },
        }
    }

    fn add_q_reg(&mut self, alias: &String, q_num: N) {
        self.q_reg.0 *= QReg::new(q_num);
        self.q_reg.1.resize(self.q_reg.1.len() + q_num, alias.clone());
    }
    fn add_c_reg(&mut self, alias: &String, q_num: N) {
        self.c_reg.0 *= CReg::new(q_num);
        self.c_reg.1.resize(self.c_reg.1.len() + q_num, alias.clone());
    }

    fn measure(&mut self, q_arg: N, c_arg: N) {
        let mask = self.q_reg.0.measure_mask(q_arg);

        match self.m_op {
            MeasureOp::Set => BitsIter::from(q_arg)
                .zip(BitsIter::from(c_arg))
                .for_each(|(q, c)| self.c_reg.0.set(mask & q != 0, c)),
            MeasureOp::Xor => BitsIter::from(q_arg)
                .zip(BitsIter::from(c_arg))
                .for_each(|(q, c)| self.c_reg.0.xor(mask & q != 0, c)),
        };
    }

    fn branch(&mut self, sep: Sep) {
        let ops = std::mem::take(&mut self.q_ops.0);
        if !ops.is_empty() {
            self.q_ops.1.push_back((ops, sep));
        }
    }

    fn branch_with_id(&mut self, sep: Sep) {
        let ops = std::mem::take(&mut self.q_ops.0);
        self.q_ops.1.push_back((ops, sep));
    }

    pub fn get_class(&self) -> CReg {
        self.c_reg.0.clone()
    }
    pub fn get_probabilities(&self) -> Vec<R> {
        self.q_reg.0.get_probabilities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regs() {
        let ast = Ast::from_source(
            "OPENQASM 2.0;\
            qreg a[1];\
            qreg b[2];\
            creg c[3];\
            creg e[4];"
        ).unwrap();
        let int = Int::new(&ast).unwrap();

        assert_eq!(int.get_q_idx(&Argument::Register("a".to_string())), Ok(1));
        assert_eq!(int.get_q_idx(&Argument::Qubit("b".to_string(), 1)), Ok(4));
        assert_eq!(int.get_q_idx(&Argument::Qubit("b".to_string(), 2)),
                   Err(Error::IdxOutOfRange( "b".to_string(), 2 )));
        assert_eq!(int.get_q_idx(&Argument::Register( "c".to_string() )),
                   Err(Error::NoQReg( "c".to_string() )));
        assert_eq!(int.get_c_idx(&Argument::Register( "c".to_string() )), Ok(7));
        assert_eq!(int.get_c_idx(&Argument::Register( "d".to_string() )),
                   Err(Error::NoCReg( "d".to_string() )));
        assert_eq!(int.get_c_idx(&Argument::Register( "e".to_string() )), Ok(120));
    }

    #[test]
    fn process_gate() {
        assert_eq!(gates::process("x".to_string(), vec![0b001, 0b100], vec![]),
                   Ok(op::x(0b101)));
        assert_eq!(gates::process("y".to_string(), vec![0b11], vec![]),
                   Ok(op::y(0b11)));
        assert_eq!(gates::process("ch".to_string(), vec![0b100, 0b010, 0b001], vec![]),
                   Ok(op::h(0b011).c(0b100)));
        assert_eq!(gates::process("swap".to_string(), vec![0b100, 0b010], vec![]),
                   Ok(op::swap(0b110)));
        assert_eq!(gates::process("swap".to_string(), vec![0b001], vec![]),
                   Err(Error::WrongRegNumber("swap".to_string(), 1)));
    }

    #[test]
    fn operation_tree() {
        let ast = Ast::from_source(
            "OPENQASM 2.0;\
            qreg q[2];\
            creg c[2];\
  \
            gate foo(x, y) a, b {\
                rx(x) a;\
            }\
\
            h q[0];\
            cx q[0], q[1];\
            foo(3.141592653589793, 0) q[0], q[1];\
\
            measure q -> c;\
\
            if (c==1) h q[0];\
            if (c==2) h q[1];\
            if (c==3) h q[0], q[1];\
\
            measure q -> c;"
        ).unwrap();
        let mut int = Int::new(&ast).unwrap();

        println!("{:#?}", int.q_ops);
    }
}