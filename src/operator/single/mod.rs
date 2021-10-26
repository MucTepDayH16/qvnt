use crate::{
    operator::atomic::*,
    math::{C, R, N},
};
pub (crate) use super::applicable::Applicable;

macro_rules! single_op_checked {
    ($op:expr) => {
        match $op {
            op if op.is_valid() => Some(op.into()),
            _ => None,
        }
    }
}

pub (crate) mod pauli;
pub (crate) mod rotate;
pub (crate) mod swap;

type Ptr<T> = std::sync::Arc<T>;

#[derive(Clone)]
pub struct SingleOp {
    pub (crate) act: N,
    pub (crate) ctrl: N,
    pub (crate) func: Ptr<dyn AtomicOp>,
}

impl SingleOp {
    pub fn name(&self) -> String {
        let mut name = self.func.name();
        if self.ctrl != 0 {
            name = format!("C{}_", self.ctrl) + &name;
        }
        name
    }
}

impl Applicable for SingleOp {
    fn apply(&self, psi: Vec<C>) -> Vec<C> {
        use rayon::iter::*;

        let len = psi.len();
        let psi = Ptr::new(psi);

        if self.ctrl != 0 {
            (0..len).into_par_iter()
                    .map(
                        |idx| if !idx & self.ctrl == 0 {self.func.atomic_op(&psi, idx)} else {psi[idx]}
                    ).collect()
        } else {
            (0..len).into_par_iter()
                    .map(
                        |idx| self.func.atomic_op(&psi, idx)
                    ).collect()
        }
    }

    #[inline]
    fn act_on(&self) -> N {
        self.act
    }

    #[inline]
    fn dgr(self) -> Self {
        Self { func: self.func.dgr(), ..self }
    }

    #[inline(always)]
    fn c(mut self, c: N) -> Option<Self> {
        if self.act & c != 0 {
            None
        } else {
            self.act |= c;
            self.ctrl |= c;
            Some(self)
        }
    }
}

impl std::fmt::Debug for SingleOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwrap_op() {
        assert!(rotate::ryy(0b001, 1.35).is_none());
        assert!(rotate::ryy(0b101, 1.35).unwrap().c(0b001).is_none());
        let _ = rotate
            ::ryy(0b101, 1.35).unwrap()
            .c(0b010).unwrap();

        assert!(swap::swap(0b001).is_none());
        assert!(swap::swap(0b101).unwrap().c(0b100).is_none());
        let _ = swap
            ::swap(0b101).unwrap()
            .c(0b010).unwrap();
    }
}