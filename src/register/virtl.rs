use {
    crate::math::{C, N, R},
    std::ops::{Index, RangeFull},
};

pub struct Reg(pub (crate) N, pub (crate) Vec<N>);

impl std::fmt::Debug for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl Index<N> for Reg {
    type Output = N;

    #[inline]
    fn index(&self, idx: N) -> &Self::Output {
        &self.1[idx]
    }
}

impl Index<RangeFull> for Reg {
    type Output = N;

    #[inline]
    fn index(&self, _: RangeFull) -> &Self::Output {
        &self.0
    }
}