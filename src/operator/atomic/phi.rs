use std::collections::BTreeMap;

use super::*;

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

impl Into<SingleOp> for Op {
    fn into(self) -> SingleOp {
        let act = self.phases.iter().fold(0, |act, (i, _)| act | *i);
        SingleOp { act, ctrl: 0, func: Ptr::new(self) }
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

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        Ptr::new(Self{ phases: self.phases.iter().map(|(idx, ang)| (*idx, ang.conj())).collect() })
    }
}