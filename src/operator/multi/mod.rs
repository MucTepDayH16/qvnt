use std::{
    collections::VecDeque,
    ops::{Mul, MulAssign},
};

pub use super::Applicable;
use crate::{math::types::*, operator::single::*};

/// Quantum operation's queue.
///
/// [`MultiOp`] is an array of [`SingleOp`](super::SingleOp)s.
/// It implements [`Deref`](std::ops::Deref) and [`DerefMut`](std::ops::DerefMut) traits
/// with ```Target = VecDeque<SingleOp>``` to inherit all methods from [`VecDeque<SingleOp>`].
///
/// As a queue, [`MultiOp`]s are able to be concatenated:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let mut first_op = op::x(0b01);
/// let mut second_op = op::y(0b10);
///
/// first_op.append(&mut second_op);
/// ```
///
/// This results to a new gate, which contains both *x* and *y* gates:
///
/// ```ignore
/// (q0) |0> -- [X]
/// (q1) |0> -- [Y]
/// ```
///
/// [`append`](VecDeque::append()) and [`push_back`](VecDeque::push_back()) was used for it.
/// However, QVNT implements [`Mul`](std::ops::Mul) and [`MulAssign`](std::ops::MulAssign) trait for [`MultiOp`]
/// to *naturify* interactions with operations, since gates are just operators in quantum mechanics and
/// could be multiplied to another operator.
/// Let's rewrite previous example in precise way:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let mut first_op = op::x(0b01);
/// let second_op = op::y(0b10);
///
/// first_op = first_op * second_op;
/// ```
///
/// Or more precise way:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let mut first_op = op::x(0b01);
/// let second_op = op::y(0b10);
///
/// first_op *= second_op;
/// ```
///
/// Or the most precise way:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let new_op = op::x(0b01) * op::y(0b10);
/// ```
#[derive(Clone, Default, PartialEq)]
pub struct MultiOp(VecDeque<SingleOp>);

impl MultiOp {
    pub fn ends_with(&self, suffix: &Self) -> bool {
        self.iter()
            .rev()
            .zip(suffix.iter().rev())
            .all(|(a, b)| a == b)
    }
}

#[doc(hidden)]
impl std::ops::Deref for MultiOp {
    type Target = VecDeque<SingleOp>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
impl std::ops::DerefMut for MultiOp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Debug for MultiOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

impl Applicable for MultiOp {
    fn apply(&self, psi_i: &[C], psi_o: &mut Vec<C>) {
        let mut psi_i = psi_i.to_vec();
        self.0.iter().for_each(|op| {
            op.apply(&psi_i, psi_o);
            std::mem::swap(&mut psi_i, psi_o);
        });
        std::mem::swap(&mut psi_i, psi_o);
    }

    #[cfg(feature = "multi-thread")]
    fn apply_sync(&self, psi_i: &[C], psi_o: &mut Vec<C>) {
        let mut psi_i = psi_i.to_vec();
        self.0.iter().for_each(|op| {
            op.apply_sync(&psi_i, psi_o);
            std::mem::swap(&mut psi_i, psi_o);
        });
        std::mem::swap(&mut psi_i, psi_o);
    }

    fn act_on(&self) -> N {
        self.0.iter().fold(0, |act, op| act | op.act_on())
    }

    fn dgr(self) -> Self {
        let new = self.0.into_iter().map(|op| op.dgr()).rev().collect();
        Self(new)
    }

    fn c(self, c_mask: N) -> Option<Self> {
        if self.act_on() & c_mask != 0 {
            None
        } else {
            let new = self.0.into_iter().map(|op| op.c(c_mask).unwrap()).collect();
            Some(Self(new))
        }
    }
}

impl From<SingleOp> for MultiOp {
    fn from(single: SingleOp) -> Self {
        Self(
            if single.name() != "Id" {
                vec![single]
            } else {
                vec![]
            }
            .into(),
        )
    }
}

impl Mul for MultiOp {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self.mul_assign(rhs);
        self
    }
}

impl Mul<SingleOp> for MultiOp {
    type Output = Self;

    fn mul(mut self, rhs: SingleOp) -> Self {
        self.mul_assign(Self::from(rhs));
        self
    }
}

impl MulAssign for MultiOp {
    fn mul_assign(&mut self, mut rhs: Self) {
        self.0.append(&mut rhs.0);
    }
}

impl MulAssign<SingleOp> for MultiOp {
    #[inline]
    fn mul_assign(&mut self, rhs: SingleOp) {
        self.mul_assign(Self::from(rhs));
    }
}

impl<'a> MulAssign<MultiOp> for &'a mut MultiOp {
    #[inline]
    fn mul_assign(&mut self, mut rhs: MultiOp) {
        self.0.append(&mut rhs.0);
    }
}

impl<'a> MulAssign<SingleOp> for &'a mut MultiOp {
    #[inline]
    fn mul_assign(&mut self, rhs: SingleOp) {
        self.mul_assign(MultiOp::from(rhs))
    }
}

pub mod h;
pub mod qft;

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn ops() {
        let pend_ops = op::id()
            * op::h(0b001).c(0b010).unwrap()
            * op::x(0b011).c(0b100).unwrap()
            * op::rz(5.0, 0b001);

        assert_eq!(pend_ops.len(), 3);
    }

    #[test]
    fn ends_with() {
        let op = (
            op::x(0x010101) * op::y(0x101010) * op::z(0x011011),
            op::y(0x101010) * op::z(0x011011),
        );

        assert!(op.1.ends_with(&op.0));
    }
}
