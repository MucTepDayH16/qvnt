use {
    std::{
        collections::VecDeque,
        ops::{Mul, MulAssign},
    },

    crate::{
        operator::single::*,
        math::{C, R, N},
    },
};

pub (crate) use super::applicable::Applicable;

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

    pub fn c(self, c: N) -> Self {
        Self(self.0.into_iter().map(|op| op.ctrl(c)).collect())
    }

    pub fn dgr(self) -> Self {
        Self(self.0.into_iter().map(|op| op.dgr()).rev().collect())
    }
}

impl std::fmt::Debug for MultiOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Applicable for MultiOp {
    #[inline(always)]
    fn apply(&self, psi: Vec<C>) -> Vec<C> {
        let iter = self.0.iter();
        crate::threads::global_install(
            move || iter.fold(psi, |psi, op| op.apply(psi))
        )
    }
}

impl From<SingleOp> for MultiOp {
    #[inline(always)]
    fn from(single: SingleOp) -> Self {
        Self(vec![single].into())
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
        self.mul_assign(rhs);
        self
    }
}

impl<'a> Mul<MultiOp> for &'a mut MultiOp {
    type Output = Self;

    fn mul(self, rhs: MultiOp) -> Self::Output {
        self.mul_assign(rhs);
        self
    }
}

impl MulAssign for MultiOp {
    fn mul_assign(&mut self, mut rhs: Self) {
        self.0.append(&mut rhs.0);
    }
}

impl MulAssign<SingleOp> for MultiOp {
    fn mul_assign(&mut self, rhs: SingleOp) {
        self.0.push_back(rhs);
    }
}

impl<'a> MulAssign<MultiOp> for &'a mut MultiOp {
    fn mul_assign(&mut self, mut rhs: MultiOp) {
        self.0.append(&mut rhs.0);
    }
}

pub (crate) mod h;
pub (crate) mod qft;