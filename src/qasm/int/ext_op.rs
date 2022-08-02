use std::{collections::VecDeque, fmt};

use crate::{
    math::{C, N, R},
    operator::{self as op, MultiOp},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Sep {
    Nop,
    Measure(N, N),
    IfBranch(N, N),
    Reset(N),
}

impl Default for Sep {
    fn default() -> Self {
        Sep::Nop
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Op(pub VecDeque<(MultiOp, Sep)>, pub MultiOp);

impl Op {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty() && self.1.is_empty()
    }

    pub(crate) fn append(&mut self, other: &mut Self) {
        let Op(ref mut vec_0, ref mut last_0) = self;
        let Op(vec_1, last_1) = std::mem::take(other);

        let mut last = std::mem::replace(last_0, last_1);
        if !last.is_empty() {
            if let Some((last_0, Sep::Nop)) = vec_0.back_mut() {
                last_0.append(&mut *last);
            } else {
                vec_0.push_back((last, Sep::Nop));
            }
        }
        vec_0.extend(vec_1);
    }

    pub(crate) fn push(&mut self, other: MultiOp) {
        if self.1.is_empty() {
            if let Some((last, Sep::Nop)) = self.0.back_mut() {
                *last *= other;
            } else {
                self.1 = other;
            }
        } else {
            self.1 *= other;
        }
    }

    pub(crate) fn ends_with(&self, suffix: &Self) -> bool {
        if suffix.0.is_empty() {
            self.1.ends_with(&suffix.1)
        } else {
            self.1 == suffix.1
                && suffix.0.iter().rev().enumerate().all(|(idx, op)| {
                    if let Some(self_op) = self.0.iter().nth_back(idx) {
                        if self_op.1 == Sep::Nop {
                            self_op.0.ends_with(&op.0)
                        } else {
                            self_op.0 == op.0
                        }
                    } else {
                        false
                    }
                })
        }
    }
}

impl std::ops::Mul for Op {
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self::Output {
        self.append(&mut rhs);
        self
    }
}

impl fmt::Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_op(op: &MultiOp) -> String {
            if !op.is_empty() {
                format!(" -> {:?}", op)
            } else {
                String::new()
            }
        }

        let mut it = self.0.iter();
        if let Some((op, sep)) = it.next() {
            match sep {
                Sep::Nop => write!(f, "{:?}", op),
                Sep::Measure(q, c) => write!(f, "{:?} -> Measure({:b} => {:b})", op, q, c),
                Sep::IfBranch(c, v) => write!(f, " -> if c[{:b}] == {:b} {{ {:?} }}", c, v, op),
                Sep::Reset(r) => write!(f, "{:?} -> Reset({:b})", op, r),
            }?;
            for (op, sep) in it {
                match sep {
                    Sep::Nop => write!(f, "{}", fmt_op(op)),
                    Sep::Measure(q, c) => {
                        write!(f, "{} -> Measure({:b} => {:b})", fmt_op(op), q, c)
                    }
                    Sep::IfBranch(c, v) => {
                        write!(f, " -> if c[{:b}] == {:b} {{ {:?} }}", c, v, op)
                    }
                    Sep::Reset(r) => write!(f, "{} -> Reset({:b})", fmt_op(op), r),
                }?;
            }

            write!(f, "{}", fmt_op(&self.1))
        } else {
            write!(f, "{:?}", self.1)
        }
    }
}

#[cfg(test)]
pub(crate) fn dummy_op() -> Op {
    Op(
        vec![
            (op::x(0b110), Sep::Nop),
            (op::h(0b111) * op::z(0b010), Sep::Measure(0, 0)),
        ]
        .into(),
        op::y(0b011),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_left() {
        let op = (
            Op(vec![(op::x(0b110), Sep::Nop)].into(), op::h(0b111)),
            Op(
                vec![(op::z(0b010), Sep::Measure(0, 0))].into(),
                op::y(0b011),
            ),
        );

        let expected = Op(
            vec![
                (op::x(0b110) * op::h(0b111), Sep::Nop),
                (op::z(0b010), Sep::Measure(0, 0)),
            ]
            .into(),
            op::y(0b011),
        );

        assert!(expected.ends_with(&op.1));
        assert_eq!(op.0 * op.1, expected);
    }

    #[test]
    fn append_right() {
        let op = (
            Op(vec![(op::x(0b110), Sep::Nop)].into(), op::h(0b111)),
            Op(
                vec![(op::z(0b010), Sep::Measure(0, 0))].into(),
                op::y(0b011),
            ),
        );

        let expected = Op(
            vec![
                (op::z(0b010), Sep::Measure(0, 0)),
                (op::y(0b011), Sep::Nop),
                (op::x(0b110), Sep::Nop),
            ]
            .into(),
            op::h(0b111),
        );

        assert!(expected.ends_with(&op.0));
        assert_eq!(op.1 * op.0, expected);
    }

    #[test]
    fn ends_with_itself() {
        let op = dummy_op();
        assert!(op.ends_with(&op));
    }
}
