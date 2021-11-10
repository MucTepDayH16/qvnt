use crate::{math::{C, N, R}, operator::single::*};
use std::{collections::VecDeque, ops::{Mul, MulAssign}};

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
#[derive(Clone)]
pub struct MultiOp(VecDeque<SingleOp>);

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
        write!(f, "{:?}", self.0)
    }
}

pub (crate) use super::Applicable;
impl Applicable for MultiOp {
    fn apply(&self, psi: Vec<C>) -> Vec<C> {
        self.0.iter()
            .fold(psi, |psi, op| op.apply(psi))
    }

    fn act_on(&self) -> N {
        self.0.iter()
            .fold(0, |act, op| act | op.act)
    }

    fn dgr(self) -> Self {
        Self(self.0.into_iter().map(|op| op.dgr()).rev().collect())
    }

    fn c(mut self, c_mask: N) -> Option<Self> {
        for op in &mut self.0 {
            *op = match op.clone().c(c_mask) {
                Some(x) => x,
                None => return None,
            };
        }
        Some(self)
    }
}

impl From<SingleOp> for MultiOp {
    fn from(single: SingleOp) -> Self {
        Self(if single.func.name() != "Id" {vec![single]} else {vec![]}.into())
    }
}

impl Default for MultiOp {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}

impl PartialEq for MultiOp {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
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

pub (crate) mod h;
pub (crate) mod qft;

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn ops() {
        let pend_ops =
            op::id() *
                op::h(0b001).c(0b010).unwrap() *
                op::x(0b011).c(0b100).unwrap() *
                op::phi(vec![(5.0, 0b001)]);

        assert_eq!(pend_ops.len(), 3);
    }
}