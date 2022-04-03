use crate::{
    math::{C, N, R},
    operator::{self as op, Applicable, MultiOp},
};
use std::{collections::VecDeque, fmt, ops::MulAssign};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Sep {
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
pub(crate) struct Op(pub VecDeque<(MultiOp, Sep)>, pub MultiOp);

impl std::ops::Mul for Op {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let Op(mut vec_0, mut last_0) = self;
        let Op(mut vec_1, mut last_1) = rhs;

        let first = match vec_1.front_mut() {
            Some(first) => &mut first.0,
            None => &mut last_1,
        };
        std::mem::swap(first, &mut last_0);
        first.append(&mut last_0);

        vec_0.extend(vec_1);

        Op(vec_0, last_1)
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

        for (op, sep) in self.0.iter() {
            match sep {
                Sep::Nop => write!(f, "{}", fmt_op(op)),
                Sep::Measure(q, c) => write!(f, "{} -> Measure({:b} => {:b})", fmt_op(op), q, c),
                Sep::IfBranch(c, v) => write!(f, " -> if c[{:b}] == {:b} {{ {:?} }}", c, v, op),
                Sep::Reset(r) => write!(f, "{} -> Reset({:b})", fmt_op(op), r),
            }?;
        }

        write!(f, "{}", fmt_op(&self.1))
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
    fn append() {
        let op = (
            Op(vec![(op::x(0b110), Sep::Nop)].into(), op::h(0b111)),
            Op(
                vec![(op::z(0b010), Sep::Measure(0, 0))].into(),
                op::y(0b011),
            ),
        );
        let expected = Op(
            vec![
                (op::x(0b110), Sep::Nop),
                (op::h(0b111) * op::z(0b010), Sep::Measure(0, 0)),
            ]
            .into(),
            op::y(0b011),
        );

        assert_eq!(op.0 * op.1, expected);
    }
}
