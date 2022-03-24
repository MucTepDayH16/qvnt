use std::collections::BTreeMap;
use super::*;

#[derive(Clone)]
pub struct Op {
    phases: BTreeMap<N, C>,
}

impl Op {
    pub fn new(phases_vec: Vec<(R, N)>) -> Self {
        let mut phases = BTreeMap::new();
        for (val, idx) in phases_vec.iter() {
            let mut jdx = 1;
            while jdx <= *idx {
                if jdx & *idx != 0 {
                    phases.entry(jdx).or_insert(crate::math::C_ZERO).im += *val;
                }
                jdx <<= 1;
            }
        }
        phases.iter_mut().for_each(|(_, val)| *val = C::from_polar(1.0, val.im));
        Self{ phases }
    }
}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut psi = psi[idx];
        for (jdx, ang) in &self.phases {
            if idx & jdx != 0 {
                psi *= ang;
            }
        }
        psi
    }

    fn name(&self) -> String {
        format!("Phase{:?}", self.phases)
    }

    fn acts_on(&self) -> N {
        self.phases.iter().fold(0, |acc, idx| acc | *idx.0)
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::Phi(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::Phi(Self{ phases: self.phases.into_iter().map(|(idx, ang)| (idx, ang.conj())).collect() })
    }
}