use std::collections::HashMap;

use super::*;

#[derive(Clone, PartialEq)]
pub struct Op {
    phases: HashMap<N, C>,
}

impl Op {
    pub fn new(phases_vec: Vec<(R, N)>) -> Self {
        let mut phases = HashMap::new();
        for (val, idx) in phases_vec.iter() {
            let mut jdx = 1;
            while jdx <= *idx {
                if jdx & *idx != 0 {
                    phases.entry(jdx).or_insert(C_ZERO).im += *val;
                }
                jdx <<= 1;
            }
        }
        phases
            .iter_mut()
            .for_each(|(_, val)| *val = C::from_polar(1.0, val.im));
        Self { phases }
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
        AtomicOpDispatch::Phi(Self {
            phases: self
                .phases
                .into_iter()
                .map(|(idx, ang)| (idx, ang.conj()))
                .collect(),
        })
    }
}
