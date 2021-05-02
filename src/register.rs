use {
    std::{
        cmp::Ordering,
        collections::{
            BTreeMap,
        },
        iter::*,
        fmt,
        mem::take,
        ops::{
            Mul,
            MulAssign,
            Index,
            RangeFull,
        },
        sync::Arc,
    },
    rand::prelude::*,
    rand_distr,
    rayon::{
        prelude::*,
        ThreadPool,
        ThreadPoolBuilder,
    },

    crate::{
        operator::*,
        types::{*, nums::*}
    }
};
use std::fmt::Debug;

pub struct VReg(N, Vec<N>);

impl fmt::Display for VReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl Index<N> for VReg {
    type Output = N;

    #[inline]
    fn index(&self, idx: N) -> &Self::Output {
        &self.1[idx]
    }
}

impl Index<RangeFull> for VReg {
    type Output = N;

    #[inline]
    fn index(&self, _: RangeFull) -> &Self::Output {
        &self.0
    }
}

#[derive(Default)]
pub struct QReg {
    psi: Vec<C>,
    q_num: N,
    q_mask: N,
    alias: Vec<u8>,
}

impl QReg {
    pub fn new(q_num: usize) -> Self {
        let q_size = N::one() << q_num;

        let mut psi = Vec::new();
        psi.resize(q_size, C::zero());
        psi[0] = C::one();

        Self {
            psi, q_num,
            q_mask: q_size.wrapping_add(!N::zero()),
            alias: Vec::new()
        }
    }

    pub fn init_state(mut self, i_state: N) -> Self {
        self.psi.iter_mut().for_each(|val| *val = C::zero());
        self.psi[self.q_mask & i_state] = C::one();
        self
    }

    pub fn set_alias_str(mut self, alias: String) -> Self {
        self.alias = Vec::from(alias);
        self.alias.resize(self.q_num, '_' as u8);
        self
    }

    pub fn set_alias_char(mut self, alias: char) -> Self {
        self.alias.resize(0, alias as u8);
        self.alias.resize(self.q_num as N, alias as u8);
        self
    }

    pub fn get_vreg(&self, c: char) -> Option<VReg> {
        let c = c as u8;
        let mut res = VReg(0, Vec::with_capacity(self.q_num));
        let mut idx = N::one();

        for &a in &self.alias {
            if a == c {
                res.0 |= idx;
                res.1.push(idx);
            }
            idx <<= 1;
        }

        if res.0 != 0 {
            Some(res)
        } else {
            None
        }
    }

    fn tensor_prod(mut self, mut other: Self) -> Self {
        let mut alias = self.alias;
        alias.append( &mut other.alias );

        let shift = (0 as u8, self.q_num as u8);
        let mask = (self.q_mask, other.q_mask);

        let self_psi = Arc::new(take(&mut self.psi));
        let other_psi = Arc::new(take(&mut other.psi));

        let q_num = self.q_num + other.q_num;
        let q_size = N::one() << q_num;
        let psi = (0..q_size)
            .into_par_iter()
            .map(
                move |idx|
                    self_psi[(idx >> shift.0) & mask.0] * other_psi[(idx >> shift.1) & mask.1]
            ).collect();

        Self {
            psi, q_num,
            q_mask: q_size.wrapping_add(!N::zero()),
            alias
        }
    }

    pub fn eval<'a>(&mut self, pend_ops: &'a mut PendingOps) -> &'a mut PendingOps {
        let len = self.psi.len();
        let mut self_psi = take(&mut self.psi);

        for Op { name: _, func } in &pend_ops.0 {
            if let Some(func) = func {
                let psi = Arc::new(&mut self_psi);

                self_psi = (0..len)
                    .into_par_iter()
                    .map_init(|| psi.clone(), |psi, idx| func(psi, idx))
                    .collect();
            }
        }

        self.psi = self_psi;
        pend_ops
    }

    fn normalize(&mut self) -> &mut Self {
        let len = self.psi.len();

        let phase = self.psi[0].conj() / self.psi[0].norm();
        let norm = self.psi.par_iter().map(|v| v.norm_sqr()).sum::<R>().sqrt().inv() * phase;

        self.psi.par_iter_mut().for_each(|v| *v *= norm);

        self
    }

    pub fn get_polar(&mut self) -> Vec<(R, R)> {
        self.psi.par_iter().map(|z| z.to_polar()).collect()
    }

    pub fn get_probabilities(&self) -> Vec::<R> {
        self.psi.par_iter().map(|z| z.norm_sqr()).collect()
    }

    pub fn measure(&mut self, mask: N) -> N {
        let mask = mask & self.q_mask;
        if mask == 0 { return 0; }

        let rand_idx = thread_rng().sample(
            rand_distr::WeightedIndex::new(
                self.get_probabilities()
            ).unwrap()
        );

        let len = self.psi.len();
        let psi = Arc::new(take(&mut self.psi));

        self.psi = (0..len)
            .into_par_iter()
            .map(move |idx|
                if (idx ^ rand_idx) & mask != 0 {
                    C::zero()
                } else {
                    psi[idx]
                }).collect();

        self.normalize();
        rand_idx & mask
    }

    pub fn sample_all(&mut self, count: N) -> Vec<N> {
        let p = self.get_probabilities();
        let c = count as f64;
        let c_sqrt = c.sqrt();

        let n = p
            .par_iter()
            .map(|&p| {
                let rnd: f64 = rand::thread_rng().sample(rand_distr::StandardNormal);
                p.sqrt() * rnd
            })
            .collect::<Vec<f64>>();

        let n_sum = n.par_iter().sum::<f64>();

        let mut n = (0..self.psi.len())
            .into_par_iter()
            .map(|idx| {
                let x = (c * p[idx] + c_sqrt * (n[idx] - n_sum * p[idx])).round() as Z;
                if x > 0 { x as N } else { 0 }
            })
            .collect::<Vec<N>>();

        let delta = n.par_iter().sum::<N>() as Z - count as Z;
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

impl fmt::Debug for QReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.psi.fmt(f)
    }
}

impl fmt::Display for QReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.alias.as_slice())
    }
}

impl Mul for QReg {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.tensor_prod(other)
    }
}

impl MulAssign for QReg {
    fn mul_assign(&mut self, rhs: Self) {
        *self = take(self).tensor_prod(rhs);
    }
}