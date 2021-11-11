use {rand::prelude::*, rand_distr};
#[cfg(feature = "cpu")]
use rayon::prelude::*;
use std::{fmt, ops::{Mul, MulAssign}};
use crate::math::*;

const MIN_QREG_LEN: usize = 8;

#[derive(Clone)]
pub struct Reg {
    pub (crate) psi: Vec<C>,
    q_num: N,
    q_mask: N,
}

impl Reg {
    pub fn new(q_num: N) -> Self {
        let q_size = 1_usize << q_num;

        let mut psi = vec![C_ZERO; q_size.max(MIN_QREG_LEN)];
        psi[0] = C_ONE;

        Self {
            psi, q_num,
            q_mask: q_size.wrapping_add(!0_usize),
        }
    }

    pub (crate) fn reset(&mut self, i_state: N) {
        self.psi = vec![C_ZERO; self.psi.len()];
        self.psi[self.q_mask & i_state] = C_ONE;
    }

    pub fn init_state(mut self, i_state: N) -> Self {
        self.reset(i_state);
        self
    }

    pub fn get_vreg(&self) -> super::VReg {
        let bi = bits_iter::BitsIter::from(self.q_mask);
        super::VReg(self.q_mask, bi.collect())
    }

    pub fn get_vreg_by_mask(&self, mask: N) -> Option<super::VReg> {
        if mask & self.q_mask == 0 {
            None
        } else {
            let bi = bits_iter::BitsIter::from(mask & self.q_mask);
            Some(super::VReg(mask, bi.collect()))
        }
    }

    // TODO: add tests for combine
    pub (crate) fn combine(q: (&Self, &Self), c: M1) -> Option<Self> {
        if q.0.q_num == q.1.q_num {
            let mut q_reg = Self::new(q.0.q_num + 1);
            let q_mask = q.0.q_mask;

            #[cfg(feature = "cpu")] let iter = q_reg.psi.par_iter_mut();
            #[cfg(not(feature = "cpu"))] let iter = q_reg.psi.iter_mut();

            iter.enumerate()
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

        #[cfg(feature = "cpu")]
        let iter = self.psi.par_iter_mut().zip(psi.par_iter());
        #[cfg(not(feature = "cpu"))]
        let iter = self.psi.iter_mut().zip(psi.iter());

        iter.for_each(|q| *q.0 = q.0.mul(c.0) + q.1.mul(c.1));
    }

    fn tensor_prod(self, other: Self) -> Self {
        let shift = (0u8, self.q_num as u8);
        let mask = (self.q_mask, other.q_mask);

        let q_num = self.q_num + other.q_num;
        let q_size = 1_usize << q_num;
        let psi = crate::threads::global_install(|| {
            #[cfg(feature = "cpu")] let iter = (0..q_size.max(MIN_QREG_LEN)).into_par_iter();
            #[cfg(not(feature = "cpu"))] let iter = (0..q_size.max(MIN_QREG_LEN)).into_iter();

            iter.map(
                    move |idx| if idx < q_size {
                        self.psi[(idx >> shift.0) & mask.0] * other.psi[(idx >> shift.1) & mask.1]
                    } else {
                        C_ZERO
                    }
                ).collect()
        });

        Self {
            psi, q_num, q_mask: q_size.wrapping_add(!0_usize),
        }
    }

    pub fn apply<Op>(&mut self, op: &Op)
        where Op: crate::operator::applicable::Applicable + Sync {
        crate::threads::global_install(|| {
            self.psi = op.apply(std::mem::take(&mut self.psi))
        });
    }

    fn normalize(&mut self) -> &mut Self {
        let norm = 1. / self.get_absolute().sqrt();
        crate::threads::global_install(|| {
            #[cfg(feature = "cpu")] let iter = self.psi.par_iter_mut();
            #[cfg(not(feature = "cpu"))] let iter = self.psi.iter_mut();

            iter.for_each(|v| *v *= norm);
        });
        self
    }

    pub fn get_polar(&self) -> Vec<(R, R)> {
        crate::threads::global_install(|| {
            #[cfg(feature = "cpu")] let iter = self.psi.par_iter();
            #[cfg(not(feature = "cpu"))] let iter = self.psi.iter();
            iter.map(|z| z.to_polar()).collect()
        })
    }

    pub fn get_probabilities(&self) -> Vec<R> {
        let abs = self.get_absolute();
        crate::threads::global_install(|| {
            #[cfg(feature = "cpu")] let iter = self.psi.par_iter();
            #[cfg(not(feature = "cpu"))] let iter = self.psi.iter();
            iter.map(|z| z.norm_sqr() / abs).collect()
        })
    }
    pub fn get_absolute(&self) -> R {
        crate::threads::global_install(|| {
            #[cfg(feature = "cpu")] let iter = self.psi.par_iter();
            #[cfg(not(feature = "cpu"))] let iter = self.psi.iter();
            iter.map(|z| z.norm_sqr()).sum()
        })
    }

    fn collapse_mask(&mut self, idy: N, mask: N) {
        crate::threads::global_install(|| {
            #[cfg(feature = "cpu")] let iter = self.psi.par_iter_mut();
            #[cfg(not(feature = "cpu"))] let iter = self.psi.iter_mut();
            iter.enumerate()
                .for_each(
                    |(idx, psi)| {
                        if (idx ^ idy) & mask != 0 {
                            *psi = C_ZERO;
                        }
                    });
        });
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

    #[cfg(feature = "cpu")]
    pub fn sample_all(&self, count: N) -> Vec<N> {
        use std::cmp::Ordering;

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
                    ((c * p[idx] + c_sqrt * (n[idx] - n_sum * p[idx])).round() as Z).max(0) as N
                })
                .collect::<Vec<N>>();

            let delta = n.par_iter().sum::<N>() as Z - count as Z;

            (n, delta)
        });
        match delta.cmp(&0) {
            Ordering::Less => {
                let delta = delta.abs() as N;
                let delta = (delta >> self.q_num, delta % self.q_mask);
                n.par_iter_mut().for_each(|n| {
                    *n += delta.0;
                });
                n.par_iter_mut().zip((0..delta.1).into_par_iter()).for_each(|(n, _)| {
                    *n += 1;
                });
            },
            Ordering::Greater => {
                let mut delta = delta as N;
                for idx in 0.. {
                    if delta == 0 { break; }
                    if n[idx & self.psi.len()] == 0 { continue; }
                    n[idx & self.psi.len()] -= 1;
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
        *self = std::mem::take(self).tensor_prod(rhs);
    }
}