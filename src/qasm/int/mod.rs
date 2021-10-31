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

    fn process_node(self, node: &AstNode) -> Result<Self> {
        match node {
            AstNode::QReg(alias, size) =>
                self.process_qreg(alias.clone(), *size as N),
            AstNode::CReg(alias, size) =>
                self.process_creg(alias.clone(), *size as N),
            AstNode::Barrier(_) =>
                self.process_barrier(),
            AstNode::Reset(_) =>
                self.process_reset(),
            AstNode::Measure(q_arg, c_arg) =>
                self.process_measure(q_arg, c_arg),
            AstNode::ApplyGate(name, regs, args) =>
                self.process_apply_gate(name, regs, args),
            AstNode::Opaque(_, _, _) =>
                self.process_opaque(),
            AstNode::Gate(name, regs, args, nodes) =>
                self.process_gate(name.clone(), regs, args, nodes),
            AstNode::If(lhs, rhs, if_block) =>
                self.process_if(lhs.clone(), *rhs as N, if_block),
        }
    }

    fn process_qreg(mut self, alias: String, q_num: N) -> Result<Self> {
        self.q_reg.0 *= QReg::new(q_num);
        self.q_reg.1.resize(self.q_reg.1.len() + q_num, alias);

        Ok(self)
    }

    fn process_creg(mut self, alias: String, q_num: N) -> Result<Self> {
        self.c_reg.0 *= CReg::new(q_num);
        self.c_reg.1.resize(self.c_reg.1.len() + q_num, alias);

        Ok(self)
    }

    fn process_barrier(self) -> Result<Self> {
        todo!("AstNode::Barrier(_)")
    }

    fn process_reset(self) -> Result<Self> {
        todo!("AstNode::Reset(_)")
    }

    fn process_measure(mut self, q_arg: &Argument, c_arg: &Argument) -> Result<Self> {
        let q_arg = self.get_q_idx(q_arg)?;
        let c_arg = self.get_c_idx(c_arg)?;
        if q_arg.count_ones() != c_arg.count_ones() {
            return Err(Error::UnmatchedRegSize(
                q_arg.count_ones() as N,
                c_arg.count_ones() as N
            ));
        }

        self.branch_with_id(Sep::Measure(q_arg, c_arg));
        Ok(self)
    }

    fn process_apply_gate(mut self, name: &String, regs: &Vec<Argument>, args: &Vec<String>) -> Result<Self> {
        if let Some(macros) = self.macros.get(name) {
            let nodes = macros.apply(name, regs, args)?;
            self.process_nodes(nodes.iter())
        } else {
            let name = name.to_lowercase();

            let regs = regs.into_iter()
                .try_fold(vec![], |mut regs, reg| {
                    regs.push(self.get_q_idx(reg)?);
                    Result::Ok(regs)
                })?;

            let args = args.into_iter()
                .try_fold(vec![], |mut args, arg| {
                    match parse::eval(arg) {
                        Some(arg) => { args.push(arg); Ok(args) },
                        None => Err(Error::UnevaluatedArgument(arg.clone())),
                    }
                })?;

            self.q_ops.0 *= gates::process(name, regs, args)?;
            Ok(self)
        }
    }

    fn process_opaque(self) -> Result<Self> {
        todo!("AstNode::Opaque(_, _, _)")
    }

    fn process_gate(mut self, name: String, regs: &Vec<String>, args: &Vec<String>, nodes: &Vec<AstNode>) -> Result<Self> {
        let macros = macros::ProcessMacro::new(
            regs.clone(),
            args.clone(),
            nodes.clone()
        )?;

        self.macros.insert(name, macros);
        Ok(self)
    }

    fn process_if(mut self, lhs: String, rhs: N, if_block: &Box<AstNode>) -> Result<Self> {
        match if_block.as_ref() {
            AstNode::ApplyGate(_, _, _) => {
                self.branch(Sep::Nop);

                self = self.process_node(if_block)?;

                let val = self.get_c_idx(&Argument::Register(lhs))?;
                self.branch(Sep::IfBranch(val, rhs));

                Ok(self)
            },
            if_block => Err(Error::DisallowedNodeInMIf(if_block.clone())),
        }
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

    fn get_q_idx(&self, arg: &Argument) -> Result<N> {
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

    fn get_c_idx(&self, arg: &Argument) -> Result<N> {
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

    pub fn get_polar_wavefunction(&self) -> Vec<(R, R)> {
        self.q_reg.0.get_polar()
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