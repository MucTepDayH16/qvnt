use crate::{
    operator::atomic::*,
    math::{C, R, N},
};
pub (crate) use super::applicable::Applicable;

type Ptr<T> = std::sync::Arc<T>;

#[derive(Clone)]
pub (crate) struct SingleOp {
    pub (crate) ctrl: N,
    pub (crate) func: Ptr<dyn AtomicOp>,
}

impl SingleOp {
    pub fn from_atomic_unchecked<A>(atomic: A) -> Self
        where A: 'static + AtomicOp {
        let func = Ptr::new(atomic);
        SingleOp{ ctrl: 0, func }
    }

    pub fn from_atomic<A>(atomic: A) -> Option<Self>
        where A: 'static + AtomicOp {
        if atomic.is_valid() {
            Some(Self::from_atomic_unchecked(atomic))
        } else {
            None
        }
    }

    pub fn name(&self) -> String {
        let mut name = self.func.name();
        if self.ctrl != 0 {
            name = format!("C{}_", self.ctrl) + &name;
        }
        name
    }

    #[inline(always)]
    pub fn ctrl(mut self, c: N) -> Self {
        self.ctrl |= c;
        self
    }

    #[inline(always)]
    pub fn dgr(mut self) -> Self {
        self.func = self.func.dgr();
        self
    }
}

impl Applicable for SingleOp {
    fn apply(&self, psi: Vec<C>) -> Vec<C> {
        if self.func.name() == "Id" {
            return psi;
        }

        use rayon::prelude::*;

        let len = psi.len();
        let psi = Ptr::new(psi);
        let SingleOp { ctrl, func } = self;

        if *ctrl != 0 {
            (0..len).into_par_iter()
                    .map_init(
                        || (psi.clone(), func.clone()),
                        |(psi, func), idx| if !idx & *ctrl == 0 {func.atomic_op(psi, idx)} else {psi[idx]})
                    .collect()
        } else {
            (0..len).into_par_iter()
                    .map_init(
                        || (psi.clone(), func.clone()),
                        |(psi, func), idx| func.atomic_op(psi, idx))
                    .collect()
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

    #[test] fn from_atomic() {
        assert!(SingleOp::from_atomic(h2::Op::new(0b01, 0b10)).is_some());
        assert!(SingleOp::from_atomic(h2::Op::new(0b01, 0b01)).is_none());
    }
}

pub (crate) mod pauli;
pub (crate) mod rotate;
pub (crate) mod swap;