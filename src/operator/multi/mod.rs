use {
    crate::{
        math::{C, N, R},
        operator::single::*,
    },
    std::{
        collections::VecDeque,
        ops::{Mul, MulAssign},
    },
};

pub (crate) use super::applicable::Applicable;

#[derive(Clone)]
pub struct MultiOp(VecDeque<SingleOp>);

impl MultiOp {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> N {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl std::fmt::Debug for MultiOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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