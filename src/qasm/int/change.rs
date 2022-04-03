use std::iter::FromIterator;

use super::{macros::Macro, ExtOp, Int, MeasureOp};

#[derive(Debug, Default, PartialEq)]
pub struct IntChange {
    m_op: bool,
    q_reg: Vec<String>,
    c_reg: Vec<String>,
    q_ops: ExtOp,
    macros: Vec<(String, Macro)>,
}

impl IntChange {
    pub fn append(mut self, mut other: Self) -> Self {
        self.m_op ^= other.m_op;
        self.q_reg.append(&mut other.q_reg);
        self.c_reg.append(&mut other.c_reg);
        self.q_ops.append(&mut other.q_ops);
        self.macros.append(&mut other.macros);
        self
    }

    pub fn apply(&self, mut int: Int) -> Int {
        if self.m_op {
            int.m_op = match int.m_op {
                MeasureOp::Set => MeasureOp::Xor,
                MeasureOp::Xor => MeasureOp::Set,
            };
        }

        if !self.q_reg.is_empty() {
            int.q_reg.0.set_num(int.q_reg.0.num() + self.q_reg.len());
            int.q_reg.1.append(&mut self.q_reg.clone());
        }

        if !self.c_reg.is_empty() {
            int.c_reg.0.set_num(int.c_reg.0.num() + self.c_reg.len());
            int.c_reg.1.append(&mut self.c_reg.clone());
        }

        if !self.q_ops.is_empty() {
            int.q_ops.append(&mut self.q_ops.clone());
        }

        if !self.macros.is_empty() {
            int.macros.extend(self.macros.clone());
        }

        int
    }

    pub fn apply_rev(&self, mut int: Int) -> Option<Int> {
        if self.m_op {
            int.m_op = match int.m_op {
                MeasureOp::Set => MeasureOp::Xor,
                MeasureOp::Xor => MeasureOp::Set,
            };
        }

        if !self.q_reg.is_empty() {
            int.q_reg.1.ends_with(&self.q_reg).then(|| ())?;
            let new_len = int.q_reg.0.num() - self.q_reg.len();
            int.q_reg.0.set_num(new_len);
            int.q_reg.1.resize(new_len, String::new());
        }

        if !self.c_reg.is_empty() {
            int.c_reg.1.ends_with(&self.c_reg).then(|| ())?;
            let new_len = int.c_reg.0.num() - self.c_reg.len();
            int.c_reg.0.set_num(new_len);
            int.c_reg.1.resize(new_len, String::new());
        }

        if !self.q_ops.is_empty() {
            int.q_ops.ends_with(&self.q_ops).then(|| ())?;
            let new_len = int.q_ops.0.len() - self.q_ops.0.len();
            int.q_ops.0.resize(new_len, Default::default());
            int.q_ops.1 = self.q_ops.1.clone();
        }

        if !self.macros.is_empty() {
            for _macro in &self.macros {
                int.macros.remove(&_macro.0);
            }
        }

        Some(int)
    }
}

#[cfg(test)]
mod tests {
    use crate::qasm::int::{ext_op, macros};

    use super::*;

    #[test]
    fn append_changes() {
        let changes = (
            IntChange {
                m_op: false,
                q_reg: vec!["q".to_string()],
                c_reg: vec!["c".to_string(), "c".to_string()],
                q_ops: ExtOp::default(),
                macros: vec![],
            },
            IntChange {
                m_op: false,
                q_reg: vec![],
                c_reg: vec![],
                q_ops: ExtOp::default(),
                macros: vec![("foo".to_string(), macros::dummy_macro())],
            },
            IntChange {
                m_op: false,
                q_reg: vec!["a".to_string(), "a".to_string()],
                c_reg: vec![],
                q_ops: ext_op::dummy_op(),
                macros: vec![],
            },
        );

        assert_eq!(
            changes.0.append(changes.1).append(changes.2).q_ops,
            IntChange {
                m_op: false,
                q_reg: vec!["q".to_string(), "a".to_string(), "a".to_string()],
                c_reg: vec!["c".to_string(), "c".to_string()],
                q_ops: ext_op::dummy_op(),
                macros: vec![("foo".to_string(), macros::dummy_macro())],
            }
            .q_ops,
        );
    }

    #[test]
    fn apply_rev_changes() {
        let changes = (
            IntChange {
                m_op: false,
                q_reg: vec!["q".to_string()],
                c_reg: vec!["c".to_string(), "c".to_string()],
                q_ops: ExtOp::default(),
                macros: vec![],
            },
            IntChange {
                m_op: false,
                q_reg: vec![],
                c_reg: vec![],
                q_ops: ExtOp::default(),
                macros: vec![("foo".to_string(), macros::dummy_macro())],
            },
            IntChange {
                m_op: false,
                q_reg: vec!["a".to_string(), "a".to_string()],
                c_reg: vec![],
                q_ops: ext_op::dummy_op(),
                macros: vec![],
            },
        );

        let mut int = Int::default();
        int = changes.0.apply(int);
        int = changes.1.apply(int);
        int = changes.2.apply(int);

        int = changes.2.apply_rev(int).unwrap();
        int = changes.1.apply_rev(int).unwrap();
        int = changes.0.apply_rev(int).unwrap();

        let _ = int;
    }
}
