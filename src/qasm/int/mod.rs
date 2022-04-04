use {
    crate::{
        math::bits_iter::BitsIter,
        math::{C, N, R},
        operator::{self as op, Applicable, MultiOp},
        qasm::ast::Ast,
        register::{CReg, QReg},
    },
    qasm::{Argument, AstNode},
    std::collections::{HashMap, VecDeque},
};

mod error;
mod ext_op;
mod gates;
mod macros;
mod parse;

use error::{Error, Result};
pub use ext_op::{Op as ExtOp, Sep};
use macros::Macro;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasureOp {
    Set,
    Xor,
}

impl Default for MeasureOp {
    fn default() -> Self {
        MeasureOp::Set
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Int {
    pub(in crate::qasm) m_op: MeasureOp,
    pub(in crate::qasm) q_reg: Vec<String>,
    pub(in crate::qasm) c_reg: Vec<String>,
    pub(in crate::qasm) q_ops: ExtOp,
    pub(in crate::qasm) macros: HashMap<String, Macro>,
}

impl Int {
    pub fn new(ast: &Ast) -> Result<Self> {
        let mut new = Int::default();
        new.add_ast(ast)?;
        Ok(new)
    }

    pub fn add_ast(&mut self, ast: &Ast) -> Result<()> {
        let mut changes = Int::default();
        self.process_nodes(&mut changes, ast.iter().cloned())?;
        unsafe { self.append_int(changes) };
        Ok(())
    }

    pub fn ast_changes(&self, ast: &Ast) -> Result<Int> {
        let mut changes = Int::default();
        self.process_nodes(&mut changes, ast.iter().cloned())?;
        Ok(changes)
    }

    pub unsafe fn append_int(&mut self, mut int: Self) {
        self.m_op = int.m_op;
        self.q_reg.append(&mut int.q_reg);
        self.c_reg.append(&mut int.c_reg);
        self.q_ops.append(&mut int.q_ops);
        self.macros.extend(int.macros.clone());
    }

    pub unsafe fn prepend_int(&mut self, mut int: Self) {
        int.append_int(std::mem::take(self));
        *self = int;
    }

    #[doc(hidden)]
    #[cfg(test)]
    pub fn as_tuple(&self) -> (&MeasureOp, &Vec<String>, &Vec<String>, &ExtOp, Vec<&String>) {
        (&self.m_op, &self.q_reg, &self.c_reg, &self.q_ops, self.macros.keys().collect())
    }

    pub fn xor(self) -> Self {
        Self {
            m_op: MeasureOp::Xor,
            ..self
        }
    }

    fn process_nodes<'a, I: Iterator<Item = AstNode>>(
        &'_ self,
        changes: &'a mut Self,
        mut nodes: I,
    ) -> Result<&'a mut Self> {
        nodes.try_fold(changes, |changes, node| self.process_node(changes, node))
    }

    fn process_node<'a>(&self, changes: &'a mut Self, node: AstNode) -> Result<&'a mut Self> {
        match node {
            AstNode::QReg(alias, size) => self.process_qreg(changes, alias, size as N),
            AstNode::CReg(alias, size) => self.process_creg(changes, alias, size as N),
            AstNode::Barrier(_) => self.process_barrier(changes),
            AstNode::Reset(ref reg) => self.process_reset(changes, reg),
            AstNode::Measure(ref q_arg, ref c_arg) => self.process_measure(changes, q_arg, c_arg),
            AstNode::ApplyGate(name, regs, args) => self.process_apply_gate(changes, name, regs, args),
            AstNode::Opaque(_, _, _) => self.process_opaque(changes),
            AstNode::Gate(name, regs, args, nodes) => self.process_gate(changes, name, regs, args, nodes),
            AstNode::If(lhs, rhs, if_block) => self.process_if(changes, lhs, rhs as N, if_block),
        }
    }

    fn process_qreg<'a>(&self, changes: &'a mut Self, alias: String, q_num: N) -> Result<&'a mut Self> {
        changes.q_reg.append(&mut vec![alias; q_num]);
        Ok(changes)
    }

    fn process_creg<'a>(&self, changes: &'a mut Self, alias: String, q_num: N) -> Result<&'a mut Self> {
        changes.c_reg.append(&mut vec![alias; q_num]);
        Ok(changes)
    }

    fn process_barrier<'a>(&self, changes: &'a mut Self) -> Result<&'a mut Self> {
        //  Does not really affect qvnt-i flow
        Ok(changes)
    }

    fn process_reset<'a>(&self, changes: &'a mut Self, q_reg: &Argument) -> Result<&'a mut Self> {
        let idx = self.get_q_idx_with_context(changes, q_reg)?;
        changes.branch_with_id(Sep::Reset(idx));
        Ok(changes)
    }

    fn process_measure<'a>(&self, changes: &'a mut Self, q_arg: &Argument, c_arg: &Argument) -> Result<&'a mut Self> {
        let q_arg = self.get_q_idx_with_context(changes, q_arg)?;
        let c_arg = self.get_c_idx_with_context(changes, c_arg)?;
        
        if q_arg.count_ones() != c_arg.count_ones() {
            return Err(Error::UnmatchedRegSize(
                q_arg.count_ones() as N,
                c_arg.count_ones() as N,
            ));
        }

        changes.branch_with_id(Sep::Measure(q_arg, c_arg));
        Ok(changes)
    }

    fn process_apply_gate<'a>(
        &self,
        changes: &'a mut Self,
        name: String,
        regs: Vec<Argument>,
        args: Vec<String>,
    ) -> Result<&'a mut Self> {
        let regs = regs
            .into_iter()
            .map(|ref reg| self.get_q_idx_with_context(changes, reg))
            .collect::<Result<Vec<_>>>()?;

        let args = args
            .into_iter()
            .map(|arg| {
                parse::eval_extended(arg.clone(), None)
                    .map_err(|e| Error::UnevaluatedArgument(arg, e))
            })
            .collect::<Result<Vec<_>>>()?;

        changes.q_ops.1 *= match self.macros.get(&name).or_else(|| changes.macros.get(&name)) {
            Some(_macro) => _macro.process(name, regs, args)?,
            None => gates::process(name.to_lowercase(), regs, args)?,
        };

        Ok(changes)
    }

    fn process_opaque<'a>(&self, changes: &'a mut Self) -> Result<&'a mut Self> {
        //  TODO: To understand what opaque gate stands for
        Ok(changes)
    }

    fn process_gate<'a>(
        &self,
        changes: &'a mut Self,
        name: String,
        regs: Vec<String>,
        args: Vec<String>,
        nodes: Vec<AstNode>,
    ) -> Result<&'a mut Self> {
        let macros = Macro::new(regs, args, nodes)?;
        if !self.macros.contains_key(&name) || !changes.macros.contains_key(&name) {
            changes.macros.insert(name, macros);
            Ok(changes)
        } else {
            Err(Error::MacroAlreadyDefined(name))
        }
    }

    fn process_if<'a>(&self, changes: &'a mut Self, lhs: String, rhs: N, if_block: Box<AstNode>) -> Result<&'a mut Self> {
        match *if_block {
            if_block @ AstNode::ApplyGate(_, _, _) => {
                changes.branch(Sep::Nop);

                let val = self.get_c_idx_with_context(changes, &Argument::Register(lhs))?;
                self.process_node(changes, if_block)?;
                changes.branch(Sep::IfBranch(val, rhs));

                Ok(changes)
            }
            if_block => Err(Error::DisallowedNodeInIf(if_block)),
        }
    }

    fn get_idx_by_alias(&self, alias: &String) -> (N, N) {
        let q_mask = self.q_reg.iter().enumerate().fold(0, |acc, (idx, name)| {
            if name == alias {
                acc | 1_usize.wrapping_shl(idx as u32)
            } else {
                acc
            }
        });
        let c_mask = self.c_reg.iter().enumerate().fold(0, |acc, (idx, name)| {
            if name == alias {
                acc | 1_usize.wrapping_shl(idx as u32)
            } else {
                acc
            }
        });

        (q_mask, c_mask)
    }

    fn get_q_idx_with_context(&self, changes: &Self, arg: &Argument) -> Result<N> {
        let self_q_len = self.q_reg.len();
        self.get_q_idx(arg)
            .or_else(|_| changes.get_q_idx(arg).map(|idx| self_q_len + idx))
    }

    fn get_q_idx(&self, arg: &Argument) -> Result<N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(alias).0;
                if mask != 0 {
                    BitsIter::from(mask)
                        .nth(*idx as N)
                        .ok_or(Error::IdxOutOfRange(alias.clone(), *idx as N))
                } else {
                    Err(Error::NoQReg(alias.clone()))
                }
            }
            Argument::Register(alias) => {
                let mask = self.get_idx_by_alias(alias).0;
                if mask != 0 {
                    Ok(mask)
                } else {
                    Err(Error::NoQReg(alias.clone()))
                }
            }
        }
    }

    fn get_c_idx_with_context(&self, changes: &Self, arg: &Argument) -> Result<N> {
        let self_c_len = self.c_reg.len();
        self.get_c_idx(arg)
            .or_else(|_| changes.get_c_idx(arg).map(|idx| self_c_len + idx))
    }

    fn get_c_idx(&self, arg: &Argument) -> Result<N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(alias).1;
                if mask != 0 {
                    BitsIter::from(mask)
                        .nth(*idx as N)
                        .ok_or(Error::IdxOutOfRange(alias.clone(), *idx as N))
                } else {
                    Err(Error::NoCReg(alias.clone()))
                }
            }
            Argument::Register(alias) => {
                let mask = self.get_idx_by_alias(alias).1;
                if mask != 0 {
                    Ok(mask)
                } else {
                    Err(Error::NoCReg(alias.clone()))
                }
            }
        }
    }

    fn branch(&mut self, sep: Sep) {
        let ops = std::mem::take(&mut self.q_ops.1);
        if !ops.is_empty() {
            self.q_ops.0.push_back((ops, sep));
        }
    }

    fn branch_with_id(&mut self, sep: Sep) {
        let ops = std::mem::take(&mut self.q_ops.1);
        self.q_ops.0.push_back((ops, sep));
    }

    // pub fn get_class(&self) -> CReg {
    //     self.c_reg.0.clone()
    // }

    // pub fn get_polar_wavefunction(&self) -> Vec<(R, R)> {
    //     self.q_reg.0.get_polar()
    // }

    // pub fn get_probabilities(&self) -> Vec<R> {
    //     self.q_reg.0.get_probabilities()
    // }

    pub fn get_ops_tree(&self) -> String {
        format!("{:?}", self.q_ops)
    }

    pub fn get_q_alias(&self) -> String {
        format!("{:?}", self.q_reg)
    }

    pub fn get_c_alias(&self) -> String {
        format!("{:?}", self.c_reg)
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
            creg e[4];",
        )
        .unwrap();
        let int = Int::new(&ast).unwrap();

        assert_eq!(int.get_q_idx(&Argument::Register("a".to_string())), Ok(1));
        assert_eq!(int.get_q_idx(&Argument::Qubit("b".to_string(), 1)), Ok(4));
        assert_eq!(
            int.get_q_idx(&Argument::Qubit("b".to_string(), 2)),
            Err(Error::IdxOutOfRange("b".to_string(), 2))
        );
        assert_eq!(
            int.get_q_idx(&Argument::Register("c".to_string())),
            Err(Error::NoQReg("c".to_string()))
        );
        assert_eq!(int.get_c_idx(&Argument::Register("c".to_string())), Ok(7));
        assert_eq!(
            int.get_c_idx(&Argument::Register("d".to_string())),
            Err(Error::NoCReg("d".to_string()))
        );
        assert_eq!(int.get_c_idx(&Argument::Register("e".to_string())), Ok(120));
    }

    #[test]
    fn operation_tree() {
        let ast = Ast::from_source(
            "OPENQASM 2.0;\
            qreg q[2];\
            creg c[2];\
  \
            gate foo(x, y) a, b {\
                rx(y*x) a;\
            }\
\
            h q[0];\
            cx q[0], q[1];\
            foo(pi, 1) q[0], q[1];\
\
            measure q -> c;\
\
            if (c==1) h q[0];\
            if (c==2) h q[1];\
            if (c==3) h q[0], q[1];\
\
            measure q -> c;",
        )
        .unwrap();
        let int = Int::new(&ast).unwrap();

        println!("{}", int.get_ops_tree());
    }
}
