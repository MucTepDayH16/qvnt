#![allow(clippy::boxed_local)]
#![allow(clippy::needless_lifetimes)]

use std::collections::HashMap;

use qasm::{Argument, AstNode};

use crate::{
    math::{bits_iter::BitsIter, count_bits, types::*},
    operator::{self as op, Applicable, MultiOp},
    qasm::ast::Ast,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeasureOp {
    #[default]
    Set,
    Xor,
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
        let mut new = Self::default();
        new.add_ast(ast)?;
        Ok(new)
    }

    pub fn add_ast(&mut self, ast: Ast<'t>) -> Result<'t, ()> {
        Self::default().ast_changes(self, ast)?;
        Ok(())
    }

    pub fn ast_changes(&self, changes: &mut Self, ast: Ast<'t>) -> Result<'t, ()> {
        match self.process_nodes(changes, ast.iter().cloned()) {
            Ok(_) => {
                changes.asts.push(ast);
                Ok(())
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

    /// # Safety
    ///
    /// Caller should ensure that appending `int`
    /// is equivalent to call `add_ast`.
    /// Otherwise could lead to unexpected interpreter flow.
    pub unsafe fn append_int(mut self, mut int: Self) -> Self {
        self.m_op = int.m_op;
        self.q_reg.append(&mut int.q_reg);
        self.c_reg.append(&mut int.c_reg);
        self.q_ops.append(&mut int.q_ops);
        self.macros.extend(int.macros.clone());
        self
    }

    /// # Safety
    ///
    /// Caller should ensure that prepending `int`
    /// is equivalent to call `add_ast`.
    /// Otherwise could lead to unexpected interpreter flow.
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
        &self,
        changes: &mut Self,
        nodes: I,
    ) -> Result<'t, ()> {
        for node in nodes {
            self.process_node(changes, node)?;
        }
        Ok(())
    }

    fn process_node(&self, changes: &mut Self, node: AstNode<'t>) -> Result<'t, ()> {
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

    #[inline]
    fn check_ident(alias: &'t str) -> Result<'t, ()> {
        let bytes_len = alias.as_bytes().len();
        if bytes_len >= 32 {
            return Err(Error::IdentIsTooLarge(alias, bytes_len));
        }
        Ok(())
    }

    #[inline]
    fn check_reg_size(alias: &'t str, q_num: N) -> Result<'t, ()> {
        if q_num >= 64 {
            return Err(Error::RegisterIsTooLarge(alias, q_num));
        }
        Ok(())
    }

    #[inline]
    fn check_dup(&self, changes: &Self, alias: &'t str) -> Result<'t, ()> {
        let count = self.q_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupQReg(alias, count));
        }

        let count = self.c_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupCReg(alias, count));
        }

        let count = changes.q_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupQReg(alias, count));
        }

        let count = changes.c_reg.iter().filter(|x| **x == alias).count();
        if count > 0 {
            return Err(Error::DupCReg(alias, count));
        }

        Ok(())
    }

    fn process_qreg(&self, changes: &mut Self, alias: &'t str, q_num: N) -> Result<'t, ()> {
        Self::check_ident(alias)?;
        Self::check_reg_size(alias, q_num)?;
        self.check_dup(changes, alias)?;
        changes.q_reg.append(&mut vec![alias; q_num]);
        Ok(())
    }

    fn process_creg(&self, changes: &mut Self, alias: &'t str, q_num: N) -> Result<'t, ()> {
        Self::check_ident(alias)?;
        Self::check_reg_size(alias, q_num)?;
        self.check_dup(changes, alias)?;
        changes.c_reg.append(&mut vec![alias; q_num]);
        Ok(())
    }

    fn process_barrier(&self, _changes: &mut Self) -> Result<'t, ()> {
        //  Does not really affect qvnt-i flow
        Ok(())
    }

    fn process_reset(&self, changes: &mut Self, q_reg: Argument<'t>) -> Result<'t, ()> {
        let idx = self.get_q_idx_with_context(changes, q_reg)?;
        changes.branch_with_id(Sep::Reset(idx));
        Ok(())
    }

    fn process_measure(
        &self,
        changes: &mut Self,
        q_arg: Argument<'t>,
        c_arg: Argument<'t>,
    ) -> Result<'t, ()> {
        let q_arg = self.get_q_idx_with_context(changes, q_arg)?;
        let c_arg = self.get_c_idx_with_context(changes, c_arg)?;

        if q_arg.count_ones() != c_arg.count_ones() {
            return Err(Error::UnmatchedRegSize(
                count_bits(q_arg),
                count_bits(c_arg),
            ));
        }

        changes.branch_with_id(Sep::Measure(q_arg, c_arg));
        Ok(())
    }

    fn process_apply_gate(
        &self,
        changes: &mut Self,
        name: &'t str,
        regs: Vec<Argument<'t>>,
        args: Vec<&'t str>,
    ) -> Result<'t, ()> {
        let regs = regs
            .into_iter()
            .map(|reg| self.get_q_idx_with_context(changes, reg))
            .collect::<Result<Vec<_>>>()?;

        let args = args
            .into_iter()
            .map(|arg| {
                parse::eval_extended(arg, None).map_err(|e| Error::UnevaluatedArgument(arg, e))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut macros = self.macros.clone();
        macros.extend(changes.macros.clone());
        let q_ops = match macros.get(name) {
            Some(_macro) => _macro.process(name, regs, args, &macros)?,
            None => gates::process(name, regs, args)?,
        };
        changes.q_ops.push(q_ops);

        Ok(())
    }

    fn process_opaque(&self, _changes: &mut Self) -> Result<'t, ()> {
        //  TODO: To understand what opaque gate stands for
        Ok(())
    }

    fn process_gate(
        &self,
        changes: &mut Self,
        name: &'t str,
        regs: Vec<&'t str>,
        args: Vec<&'t str>,
        nodes: Vec<AstNode<'t>>,
    ) -> Result<'t, ()> {
        let macros = Macro::new(regs, args, nodes)?;
        if !self.macros.contains_key(&name) && !changes.macros.contains_key(&name) {
            Self::check_ident(name)?;
            changes.macros.insert(name, macros);
            Ok(())
        } else {
            Err(Error::MacroAlreadyDefined(name))
        }
    }

    fn process_if(
        &self,
        changes: &mut Self,
        lhs: &'t str,
        rhs: N,
        if_block: Box<AstNode<'t>>,
    ) -> Result<'t, ()> {
        match *if_block {
            if_block @ AstNode::ApplyGate(_, _, _) => {
                changes.branch(Sep::Nop);

                let val = self.get_c_idx_with_context(changes, Argument::Register(lhs))?;
                self.process_node(changes, if_block)?;
                changes.branch(Sep::IfBranch(val, rhs));

                Ok(())
            }
            if_block => Err(Error::DisallowedNodeInIf(if_block)),
        }
    }

    fn get_idx_by_alias(&self, changes: &Self, alias: &'t str) -> (N, N) {
        fn fold_idx_by_alias<'a>(iter: impl Iterator<Item = &'a str>, alias: &str) -> usize {
            iter.enumerate()
                .filter(|(_, name)| *name == alias)
                .fold(0, |acc, (idx, _)| acc | 1_usize.wrapping_shl(idx as u32))
        }

        let q_mask = fold_idx_by_alias(self.q_reg.iter().chain(&changes.q_reg).cloned(), alias);
        let c_mask = fold_idx_by_alias(self.c_reg.iter().chain(&changes.c_reg).cloned(), alias);

        (q_mask, c_mask)
    }

    fn get_q_idx_with_context(&self, changes: &Self, arg: Argument<'t>) -> Result<'t, N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(changes, alias).0;
                if mask != 0 {
                    BitsIter::from(mask)
                        .nth(idx as N)
                        .ok_or(Error::IdxOutOfRange(alias, idx as N))
                } else {
                    Err(Error::NoQReg(alias))
                }
            }
            Argument::Register(alias) => {
                let mask = self.get_idx_by_alias(changes, alias).0;
                if mask != 0 {
                    Ok(mask)
                } else {
                    Err(Error::NoQReg(alias))
                }
            }
        }
    }

    fn get_q_idx(&self, arg: Argument<'t>) -> Result<'t, N> {
        self.get_q_idx_with_context(&Default::default(), arg)
    }

    fn get_c_idx_with_context(&self, changes: &Self, arg: Argument<'t>) -> Result<'t, N> {
        match arg {
            Argument::Qubit(alias, idx) => {
                let mask = self.get_idx_by_alias(changes, alias).1;
                if mask != 0 {
                    BitsIter::from(mask)
                        .nth(idx as N)
                        .ok_or(Error::IdxOutOfRange(alias, idx as N))
                } else {
                    Err(Error::NoCReg(alias))
                }
            }
            Argument::Register(alias) => {
                let mask = self.get_idx_by_alias(changes, alias).1;
                if mask != 0 {
                    Ok(mask)
                } else {
                    Err(Error::NoCReg(alias))
                }
            }
        }
    }

    fn get_c_idx(&self, arg: Argument<'t>) -> Result<'t, N> {
        self.get_c_idx_with_context(&Default::default(), arg)
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
        assert_ne!(int.asts.len(), 0);
    }

    fn int_from_source(source: &'static str) -> Result<Int> {
        let ast = Ast::from_source(source).unwrap();
        Int::new(ast)
    }

    #[test]
    fn no_qreg() {
        assert_eq!(int_from_source("h q[2];"), Err(Error::NoQReg("q")),);
        assert!(int_from_source("qreg q[5]; h q[2];").is_ok());
    }

    #[test]
    fn no_creg() {
        assert_eq!(
            int_from_source("qreg q[2]; measure q -> c;"),
            Err(Error::NoCReg("c")),
        );
        assert!(int_from_source("qreg q[2]; creg c[2]; measure q -> c;").is_ok());
    }

    #[test]
    fn dub_register() {
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
    }

    #[test]
    fn out_of_range() {
        assert_eq!(
            int_from_source("qreg q[2]; h q[2];"),
            Err(Error::IdxOutOfRange("q", 2)),
        );
        assert_eq!(
            int_from_source("qreg q[2]; h q[3];"),
            Err(Error::IdxOutOfRange("q", 3)),
        );
        assert!(int_from_source("qreg q[2]; h q[1];").is_ok());
    }

    #[test]
    fn unknown_gate() {
        assert_eq!(
            int_from_source("qreg q[3]; g q[2];"),
            Err(Error::UnknownGate("g")),
        );
    }

    #[test]
    fn invalid_ctrl_mask() {
        assert_eq!(
            int_from_source("qreg q[4]; cx q[0], q;"),
            Err(Error::InvalidControlMask(0b0001, 0b1111)),
        );
    }

    #[test]
    fn evaluation_error() {
        assert_eq!(
            int_from_source("qreg q[4]; rx(2*a) q[0];"),
            Err(Error::UnevaluatedArgument(
                "2*a",
                meval::Error::UnknownVariable("a".to_string())
            )),
        );
    }

    #[test]
    fn wrong_number() {
        assert_eq!(
            int_from_source("qreg q[4]; rx(pi) q[0], q[2];"),
            Err(Error::WrongRegNumber("rx", 2)),
        );

        assert_eq!(
            int_from_source("qreg q[4]; rx(pi, 2*pi) q[0];"),
            Err(Error::WrongArgNumber("rx", 2)),
        );
    }

    #[test]
    fn unmatched_size() {
        assert_eq!(
            int_from_source("qreg q[4]; creg c[3]; measure q -> c;"),
            Err(Error::UnmatchedRegSize(4, 3)),
        );
    }

    #[test]
    fn macro_already_defined() {
        assert_eq!(
            int_from_source("gate m q { h q; }  gate m q { x q; }"),
            Err(Error::MacroAlreadyDefined("m")),
        );
    }

    #[test]
    fn bad_op_in_if_block() {
        assert_eq!(
            int_from_source("qreg q[1]; creg c[1]; if (c==1) measure q -> c;"),
            Err(Error::DisallowedNodeInIf(AstNode::Measure(
                Argument::Register("q"),
                Argument::Register("c")
            )))
        );
    }

    #[test]
    fn invalid_ident() {
        assert_eq!(
            int_from_source(
                "qreg AAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaa[1];"
            ),
            Err(Error::IdentIsTooLarge(
                "AAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaaAAaaaaaaaaa",
                66
            ))
        );
    }

    #[test]
    fn invalid_size() {
        assert_eq!(
            int_from_source("qreg q[64];"),
            Err(Error::RegisterIsTooLarge("q", 64))
        );
    }

    #[test]
    fn index_register_in_macro() {
        assert_eq!(
            int_from_source("gate M q { h q[0]; }"),
            Err(Error::MacroError(macros::Error::DisallowedRegister("q", 0))),
        );
    }

    #[test]
    fn unknown_reg() {
        assert_eq!(
            int_from_source("gate M(a, b, c) x, y { rx(a) x; ry(b) y; rz(c) z; }"),
            Err(Error::MacroError(macros::Error::UnknownReg("z")))
        );
    }

    #[test]
    fn unknown_arg() {
        assert_eq!(
            int_from_source("gate M(a, b) x, y, z { rx(a) x; ry(b) y; rz(c) z; }"),
            Err(Error::MacroError(macros::Error::UnknownArg(
                "c".to_string()
            )))
        );
    }
}
