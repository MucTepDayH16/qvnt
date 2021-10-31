use {
    std::{
        cmp::Ordering,
        fmt,
        mem::{replace, take},
        ops::{
            Mul,
            MulAssign,
        },
        sync::Arc,
    },
    rand::prelude::*,
    rand_distr,
    rayon::prelude::*,
    
    crate::math::*,
};

use super::VReg;

const MIN_QREG_LEN: usize = 8;

#[derive(Clone)]
pub struct Reg {
    pub(crate) psi: Vec<C>,
    q_num: N,
    q_mask: N,
}

impl Reg {
    pub fn new(q_num: N) -> Self {
        let q_size = 1_usize << q_num;

        let mut psi = Vec::new();
        psi.resize(q_size.max(MIN_QREG_LEN), C_ZERO);
        psi[0] = C_ONE;

        Self {
            psi, q_num,
            q_mask: q_size.wrapping_add(!0_usize),
        }
    }

    pub (crate) fn reset(&mut self, i_state: N) {
        let mut self_psi = take(&mut self.psi);
        self.psi = crate::threads::global_install(move || {
            self_psi.par_iter_mut().for_each(|val| *val = C_ZERO);
            self_psi
        });
        self.psi[self.q_mask & i_state] = C_ONE;
    }

    pub fn init_state(mut self, i_state: N) -> Self {
        self.reset(i_state);
        self
    }

    pub fn get_vreg(&self) -> VReg {
        let mut res = VReg(self.q_mask, Vec::with_capacity(self.q_num));

        for idx in 0..self.q_num {
            res.1.push(1 << idx);
        }

        res
    }

    pub fn get_vreg_by_mask(&self, mask: N) -> Option<VReg> {
        if mask == 0 {
            None
        } else {
            let mut vec = Vec::with_capacity(mask.count_ones() as usize);

            for idx in 0..self.q_num {
                let jdx = 1 << idx;
                if jdx & mask != 0 {
                    vec.push(jdx);
                }
            }

            Some(VReg(mask, vec))
        }
    }

    // TODO: add tests for combine
    pub (crate) fn combine(q: (&Self, &Self), c: M1) -> Option<Self> {
        if q.0.q_num == q.1.q_num {
            let mut q_reg = Self::new(q.0.q_num + 1);
            let q_mask = q.0.q_mask;
            //  let mid = 1 << q.0.q_num;
            //  q_reg.psi[..mid].clone_from_slice(&q.0.psi);
            //  q_reg.psi[mid..].clone_from_slice(&q.1.psi);
            q_reg.psi.par_iter_mut()
                .enumerate()
                .for_each(|(idx, v)| {
                    let q = (q.0.psi[q_mask & idx], q.1.psi[q_mask & idx]);
                    if !q_mask & idx == 0 {
                        *v = c[0b00] * q.0 + c[0b01] * q.1;
                    } else {
                        *v = c[0b10] * q.0 + c[0b11] * q.1;
                    }
                });
            Some(q_reg)
        } else {
            None
        }
    }

    // TODO: add tests for linear_composition
    pub (crate) fn linear_composition(&mut self, psi: &[C], c: (C, C)) {
        assert_eq!(self.psi.len(), psi.len());

        self.psi.par_iter_mut()
            .zip(psi.par_iter())
            .for_each(|q| *q.0 = q.0.mul(c.0) + q.1.mul(c.1));
    }

    fn tensor_prod(mut self, mut other: Self) -> Self {
        let shift = (0 as u8, self.q_num as u8);
        let mask = (self.q_mask, other.q_mask);

        let self_psi = Arc::new(take(&mut self.psi));
        let other_psi = Arc::new(take(&mut other.psi));

        let q_num = self.q_num + other.q_num;
        let q_size = 1_usize << q_num;
        let psi: Vec<C> = crate::threads::global_install(|| {
            (0..q_size.max(MIN_QREG_LEN))
                .into_par_iter()
                .map(
                    move |idx| if idx < q_size {
                        self_psi[(idx >> shift.0) & mask.0] * other_psi[(idx >> shift.1) & mask.1]
                    } else {
                        C_ZERO
                    }
                ).collect()
        });

        Self {
            psi, q_num,
            q_mask: q_size.wrapping_add(!0_usize),
        }
    }

    pub fn apply(&mut self, ops: &impl crate::operator::applicable::Applicable) {
        self.psi = ops.apply(take(&mut self.psi));
    }

    fn normalize(&mut self) -> &mut Self {
        let norm = 1. / self.get_absolute().sqrt();
        let mut self_psi = take(&mut self.psi);
        self.psi = crate::threads::global_install(move || {
            self_psi.par_iter_mut().for_each(|v| *v *= norm);
            self_psi
        });
        self
    }

    pub fn get_polar(&self) -> Vec<(R, R)> {
        crate::threads::global_install(|| {
            self.psi[..(1_usize << self.q_num)].par_iter().map(|z| z.to_polar()).collect()
        })
    }

    pub fn get_probabilities(&self) -> Vec<R> {
        crate::threads::global_install(|| {
            let abs = self.get_absolute();
            self.psi[..(1_usize << self.q_num)].par_iter().map(|z| z.norm_sqr() / abs).collect()
        })
    }
    pub fn get_absolute(&self) -> R {
        crate::threads::global_install(|| {
            self.psi.par_iter().map(|z| z.norm_sqr()).sum()
        })
    }

    fn collapse_mask(&mut self, idy: N, mask: N) {
        let len = self.psi.len();
        let psi = Arc::new(take(&mut self.psi));

        self.psi = crate::threads::global_install(|| (0..len)
            .into_par_iter()
            .map_init(
                || psi.clone(),
                move |psi, idx|
                    if (idx ^ idy) & mask != 0 {
                        C_ZERO
                    } else {
                        psi[idx]
                    }
            ).collect()
        );
    }
    pub fn measure_mask(&mut self, mask: N) -> N {
        let mask = mask & self.q_mask;
        if mask == 0 { return 0; }

        let rand_idx = thread_rng().sample(
            rand_distr::WeightedIndex::new(
                self.get_probabilities()
            ).unwrap()
        );

        self.collapse_mask(rand_idx, mask);
        rand_idx & mask
    }
    pub fn measure(&mut self) -> N {
        self.measure_mask(self.q_mask)
    }

    pub fn sample_all(&self, count: N) -> Vec<N> {
        let p = self.get_probabilities();
        let c = count as R;
        let c_sqrt = c.sqrt();

        let (mut n, delta) = crate::threads::global_install(|| {
            let n = p
                .par_iter()
                .map(|&p| {
                    let rnd: R = rand::thread_rng().sample(rand_distr::StandardNormal);
                    p.sqrt() * rnd
                })
                .collect::<Vec<R>>();

            let n_sum = n.par_iter().sum::<R>();

            let n = (0..self.psi.len())
                .into_par_iter()
                .map(|idx| {
                    let x = (c * p[idx] + c_sqrt * (n[idx] - n_sum * p[idx])).round() as Z;
                    if x > 0 { x as N } else { 0 }
                })
                .collect::<Vec<N>>();

            let delta = n.par_iter().sum::<N>() as Z - count as Z;

            (n, delta)
        });
        match delta.cmp(&0) {
            Ordering::Less => {
                for idx in 0..(delta.abs() as N) {
                    n[idx] += 1;
                }
            },
            Ordering::Greater => {
                let mut delta = delta as N;
                for idx in 0.. {
                    if delta == 0 { break; }
                    if n[idx] <= 0 { continue; }
                    n[idx] -= 1;
                    delta -= 1;
                }
            },
            _ => {},
        }

        n
    }
}

impl Default for Reg {
    fn default() -> Self {
        Self::new(0)
    }
}

impl fmt::Debug for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.psi[..MIN_QREG_LEN].fmt(f)
    }
}

impl Mul for Reg {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.tensor_prod(other)
    }
}

impl MulAssign for Reg {
    fn mul_assign(&mut self, rhs: Self) {
        *self = replace(self, Self{ psi: vec![], q_num: 0, q_mask: 0 }).tensor_prod(rhs);
    }
}