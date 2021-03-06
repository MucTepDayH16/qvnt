use {
    crate::{
        math::bits_iter::BitsIter,
        math::{C, N, R},
        operator::{self as op, Applicable, MultiOp},
        qasm::ast::Ast,
    },
    qasm::{Argument, AstNode},
    std::collections::HashMap,
};

mod error;
mod ext_op;
mod gates;
pub mod macros;
mod parse;

use std::fmt;

pub use error::{Error, Result};
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

#[derive(Clone, Default, PartialEq)]
pub struct Int<'t> {
    pub(in crate::qasm) m_op: MeasureOp,
    pub(in crate::qasm) q_reg: Vec<&'t str>,
    pub(in crate::qasm) c_reg: Vec<&'t str>,
    pub(in crate::qasm) q_ops: ExtOp,
    pub(in crate::qasm) macros: HashMap<&'t str, Macro<'t>>,
    pub(in crate::qasm) asts: Vec<Ast<'t>>,
}

impl<'t> fmt::Debug for Int<'t> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Int")
            .field("m_op", &self.m_op)
            .field("q_reg", &self.q_reg)
            .field("c_reg", &self.c_reg)
            .field("q_ops", &self.q_ops)
            .field("macros", &self.macros)
            .finish_non_exhaustive()
    }
}

impl<'t> Int<'t> {
    pub fn new(ast: Ast<'t>) -> Result<'t, Self> {
        Self::default().add_ast(ast)
    }

    pub fn add_ast(mut self, ast: Ast<'t>) -> Result<'t, Self> {
        Self::default().ast_changes(&mut self, ast)?;
        Ok(self)
    }

    pub fn ast_changes(self, changes: &mut Self, ast: Ast<'t>) -> Result<'t, Self> {
        match self.process_nodes(changes, ast.iter().cloned()) {
            Ok(mut ok) => {
                ok.asts.push(ast);
                Ok(ok)
            }
            Err(err) => Err(err),
        }
    }

    pub fn iter_ast(&self) -> impl Iterator<Item = &Ast<'t>> {
        self.asts.iter()
    }

    pub fn into_iter_ast(self) -> impl Iterator<Item = Ast<'t>> {
        self.asts.into_iter()
    }

    pub unsafe fn append_int(mut self, mut int: Self) -> Self {
        self.m_op = int.m_op;
        self.q_reg.append(&mut int.q_reg);
        self.c_reg.append(&mut int.c_reg);
        self.q_ops.append(&mut int.q_ops);
        self.macros.extend(int.macros.clone());
        self
    }

    pub unsafe fn prepend_int(self, int: Self) -> Self {
        int.append_int(self)
    }

    pub fn xor(self) -> Self {
        Self {
            m_op: MeasureOp::Xor,
            ..self
        }
    }

    fn process_nodes<'a, I: IntoIterator<Item = AstNode<'t>>>(
        mut self,
        changes: &mut Self,
        nodes: I,
    ) -> Result<'t, Self> {
        for node in nodes {
            self = self.process_node(changes, node)?;
        }
        Ok(self)
    }

    fn process_node<'a>(self, changes: &'a mut Self, node: AstNode<'t>) -> Result<'t, Self> {
        match node {
            AstNode::QReg(alias, size) => self.process_qreg(changes, alias, size as N),
            AstNode::CReg(alias, size) => self.process_creg(changes, alias, size as N),
            AstNode::Barrier(_) => self.process_barrier(changes),
            AstNode::Reset(reg) => self.process_reset(changes, reg),
            AstNode::Measure(q_arg, c_arg) => self.process_measure(changes, q_arg, c_arg),
            AstNode::ApplyGate(name, regs, args) => {
                self.process_apply_gate(changes, name, regs, args)
            }
            AstNode::Opaque(_, _, _) => self.process_opaque(changes),
            AstNode::Gate(name, regs, args, nodes) => {
                self.process_gate(changes, name, regs, args, nodes)
            }
            AstNode::If(lhs, rhs, if_block) => self.process_if(changes, lhs, rhs as N, if_block),
        }
    }

    fn check_dup(&self, changes: &Self, alias: &'t str) -> Result<'t, ()> {
        let count = self.q_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupQReg(alias, count));
        }

        let count = self.c_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupCReg(alias.clone(), count));
        }

        let count = changes.q_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupQReg(alias.clone(), count));
        }

        let count = changes.c_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupCReg(alias.clone(), count));
        }

        Ok(())
    }

    fn process_qreg<'a>(self, changes: &mut Self, alias: &'t str, q_num: N) -> Result<'t, Self> {
        self.check_dup(changes, &alias)?;
        changes.q_reg.append(&mut vec![alias; q_num]);
        Ok(self)
    }

    fn process_creg<'a>(self, changes: &mut Self, alias: &'t str, q_num: N) -> Result<'t, Self> {
        self.check_dup(changes, &alias)?;
        changes.c_reg.append(&mut vec![alias; q_num]);
        Ok(self)
    }

    fn process_barrier(self, _changes: &mut Self) -> Result<'t, Self> {
        //  Does not really affect qvnt-i flow
        Ok(self)
    }

    fn process_reset(self, changes: &mut Self, q_reg: Argument<'t>) -> Result<'t, Self> {
        let idx = self.get_q_idx_with_context(changes, q_reg)?;
        changes.branch_with_id(Sep::Reset(idx));
        Ok(self)
    }

    fn process_measure<'a>(
        self,
        changes: &mut Self,
        q_arg: Argument<'t>,
        c_arg: Argument<'t>,
    ) -> Result<'t, Self> {
        let q_arg = self.get_q_idx_with_context(changes, q_arg)?;
        let c_arg = self.get_c_idx_with_context(changes, c_arg)?;

        if q_arg.count_ones() != c_arg.count_ones() {
            return Err(Error::UnmatchedRegSize(
                q_arg.count_ones() as N,
                c_arg.count_ones() as N,
            ));
        }

        changes.branch_with_id(Sep::Measure(q_arg, c_arg));
        Ok(self)
    }

    fn process_apply_gate<'a>(
        self,
        changes: &mut Self,
        name: &'t str,
        regs: Vec<Argument<'t>>,
        args: Vec<&'t str>,
    ) -> Result<'t, Self> {
        let regs = regs
            .into_iter()
            .map(|reg| self.get_q_idx_with_context(changes, reg))
            .collect::<Result<Vec<_>>>()?;

        let args = args
            .into_iter()
            .map(|arg| {
                parse::eval_extended(arg.clone(), None)
                    .map_err(|e| Error::UnevaluatedArgument(arg, e))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut macros = self.macros.clone();
        macros.extend(changes.macros.clone());
        let q_ops = match macros.get(name) {
            Some(_macro) => _macro.process(name, regs, args, &macros)?,
            None => gates::process(name, regs, args)?,
        };
        changes.q_ops.push(q_ops);

        Ok(self)
    }

    fn process_opaque(self, _changes: &mut Self) -> Result<'t, Self> {
        //  TODO: To understand what opaque gate stands for
        Ok(self)
    }

    fn process_gate(
        self,
        changes: &mut Self,
        name: &'t str,
        regs: Vec<&'t str>,
        args: Vec<&'t str>,
        nodes: Vec<AstNode<'t>>,
    ) -> Result<'t, Self> {
        let macros = Macro::new(regs, args, nodes)?;
        if !self.macros.contains_key(&name) && !changes.macros.contains_key(&name) {
            changes.macros.insert(name, macros);
            Ok(self)
        } else {
            Err(Error::MacroAlreadyDefined(name))
        }
    }

    fn process_if(
        mut self,
        changes: &mut Self,
        lhs: &'t str,
        rhs: N,
        if_block: Box<AstNode<'t>>,
    ) -> Result<'t, Self> {
        match *if_block {
            if_block @ AstNode::ApplyGate(_, _, _) => {
                changes.branch(Sep::Nop);

                let val = self.get_c_idx_with_context(changes, Argument::Register(lhs))?;
                self = self.process_node(changes, if_block)?;
                changes.branch(Sep::IfBranch(val, rhs));

                Ok(self)
            }
            if_block => Err(Error::DisallowedNodeInIf(if_block)),
        }
    }

    fn get_idx_by_alias(&self, alias: &'t str) -> (N, N) {
        let q_mask = self
            .q_reg
            .iter()
            .cloned()
            .enumerate()
            .fold(0, |acc, (idx, name)| {
                if name == alias {
                    acc | 1_usize.wrapping_shl(idx as u32)
                } else {
                    acc
                }
            });
        let c_mask = self
            .c_reg
            .iter()
            .cloned()
            .enumerate()
            .fold(0, |acc, (idx, name)| {
                if name == alias {
                    acc | 1_usize.wrapping_shl(idx as u32)
                } else {
                    acc
                }
            });

        (q_mask, c_mask)
    }

    fn get_q_idx_with_context(&self, changes: &Self, arg: Argument<'t>) -> Result<'t, N> {
        let self_q_len = self.q_reg.len();
        self.get_q_idx(arg.clone())
            .or_else(|_| changes.get_q_idx(arg).map(|idx| self_q_len + idx))
    }

    fn get_q_idx(&self, arg: Argument<'t>) -> Result<'t, N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(alias).0;
                if mask != 0 {
                    BitsIter::from(mask)
                        .nth(idx as N)
                        .ok_or(Error::IdxOutOfRange(alias, idx as N))
                } else {
                    Err(Error::NoQReg(alias.clone()))
                }
            }
            Argument::Register(alias) => {
                let mask = self.get_idx_by_alias(alias).0;
                if mask != 0 {
                    Ok(mask)
                } else {
                    Err(Error::NoQReg(alias))
                }
            }
        }
    }

    fn get_c_idx_with_context(&self, changes: &Self, arg: Argument<'t>) -> Result<'t, N> {
        let self_c_len = self.c_reg.len();
        self.get_c_idx(arg.clone())
            .or_else(|_| changes.get_c_idx(arg).map(|idx| self_c_len + idx))
    }

    fn get_c_idx(&self, arg: Argument<'t>) -> Result<'t, N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(alias).1;
                if mask != 0 {
                    BitsIter::from(mask)
                        .nth(idx as N)
                        .ok_or(Error::IdxOutOfRange(alias.clone(), idx as N))
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
        let int = Int::new(ast).unwrap();

        assert_eq!(int.get_q_idx(Argument::Register("a")), Ok(1));
        assert_eq!(int.get_q_idx(Argument::Qubit("b", 1)), Ok(4));
        assert_eq!(
            int.get_q_idx(Argument::Qubit("b", 2)),
            Err(Error::IdxOutOfRange("b", 2))
        );
        assert_eq!(
            int.get_q_idx(Argument::Register("c")),
            Err(Error::NoQReg("c"))
        );
        assert_eq!(int.get_c_idx(Argument::Register("c")), Ok(7));
        assert_eq!(
            int.get_c_idx(Argument::Register("d")),
            Err(Error::NoCReg("d"))
        );
        assert_eq!(int.get_c_idx(Argument::Register("e")), Ok(120));
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
        let int = Int::new(ast).unwrap();

        println!("{}", int.get_ops_tree());
    }

    #[test]
    fn check_for_errors() {
        fn int_from_source(source: &'static str) -> Result<Int> {
            let ast = Ast::from_source(source).unwrap();
            Int::new(ast)
        }

        assert_eq!(int_from_source("h q[2];"), Err(Error::NoQReg("q")),);

        assert_eq!(
            int_from_source("qreg q[2]; measure q -> c;"),
            Err(Error::NoCReg("c")),
        );

        assert_eq!(
            int_from_source("qreg q[3]; qreg q[2];"),
            Err(Error::DupQReg("q", 3)),
        );

        assert_eq!(
            int_from_source("creg c[5]; creg c[1];"),
            Err(Error::DupCReg("c", 5)),
        );

        assert_eq!(
            int_from_source("qreg x[5]; creg x[1];"),
            Err(Error::DupQReg("x", 5)),
        );

        assert_eq!(
            int_from_source("creg x[1]; qreg x[5];"),
            Err(Error::DupCReg("x", 1)),
        );

        assert_eq!(
            int_from_source("qreg q[2]; h q[2];"),
            Err(Error::IdxOutOfRange("q", 2)),
        );

        assert_eq!(
            int_from_source("qreg q[3]; g q[2];"),
            Err(Error::UnknownGate("g")),
        );

        assert_eq!(
            int_from_source("qreg q[4]; cx q[0], q;"),
            Err(Error::InvalidControlMask(0b0001, 0b1111)),
        );

        assert_eq!(
            int_from_source("qreg q[4]; rx(2*a) q[0];"),
            Err(Error::UnevaluatedArgument(
                "2*a",
                meval::Error::UnknownVariable("a".to_string())
            )),
        );

        assert_eq!(
            int_from_source("qreg q[4]; rx(pi) q[0], q[2];"),
            Err(Error::WrongRegNumber("rx", 2)),
        );

        assert_eq!(
            int_from_source("qreg q[4]; rx(pi, 2*pi) q[0];"),
            Err(Error::WrongArgNumber("rx", 2)),
        );

        assert_eq!(
            int_from_source("qreg q[4]; creg c[3]; measure q -> c;"),
            Err(Error::UnmatchedRegSize(4, 3)),
        );

        assert_eq!(
            int_from_source("gate M q { h q[0]; }"),
            Err(Error::MacroError(macros::Error::DisallowedRegister("q", 0))),
        );

        assert_eq!(
            int_from_source("gate M(a, b, c) x, y { rx(a) x; ry(b) y; rz(c) z; }"),
            Err(Error::MacroError(macros::Error::UnknownReg("z")))
        );

        assert_eq!(
            int_from_source("gate M(a, b) x, y, z { rx(a) x; ry(b) y; rz(c) z; }"),
            Err(Error::MacroError(macros::Error::UnknownArg(
                "c".to_string()
            )))
        );

        assert_eq!(
            int_from_source("gate m q { h q; }  gate m q { x q; }"),
            Err(Error::MacroAlreadyDefined("m")),
        );

        assert_eq!(
            int_from_source("qreg q[1]; creg c[1]; if (c==1) measure q -> c;"),
            Err(Error::DisallowedNodeInIf(AstNode::Measure(
                Argument::Register("q"),
                Argument::Register("c")
            )))
        );
    }
}
