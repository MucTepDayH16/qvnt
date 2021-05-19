use {
    std::{
        cmp::Ordering,
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
    rayon::prelude::*,

    crate::{
        math::*,
        operator::*,
    }
};

pub struct VReg(N, Vec<N>);

impl fmt::Debug for VReg {
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

const MIN_QREG_LEN: usize = 8;

#[derive(Default, Clone)]
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
        psi.resize(q_size.max(MIN_QREG_LEN), C::zero());
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

    pub fn get_vreg_by_char(&self, c: char) -> Option<VReg> {
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

    pub fn apply(&mut self, ops: &Op) {
        let len = self.psi.len();
        let mut self_psi = take(&mut self.psi);

        for Operator { name: _, control, func } in &ops.0 {
            let psi = Arc::new(self_psi);
            let c_mask = *control.clone();
            let par_iter = (0..len).into_par_iter();

            self_psi = if c_mask != 0 {
                par_iter
                    .map_init(|| (c_mask, psi.clone()),
                              |(c_mask, psi), idx| if !idx & *c_mask == 0 { func(psi, idx) } else { psi[idx] })
                    .collect()
            } else {
                par_iter
                    .map_init(|| psi.clone(),
                              |psi, idx| func(psi, idx))
                    .collect()
            };
        }

        self.psi = self_psi;
    }

    fn normalize(&mut self) -> &mut Self {
        let norm = self.psi.par_iter().map(|v| v.norm_sqr()).sum::<R>().sqrt().inv();
        self.psi.par_iter_mut().for_each_with(norm, |n, v| *v *= *n);
        self
    }

    pub fn get_polar(&mut self) -> Vec<(R, R)> {
        self.psi.par_iter().map(|z| z.to_polar()).collect()
    }

    pub fn get_probabilities(&self) -> Vec::<R> {
        self.psi.par_iter().map(|z| z.norm_sqr()).collect()
    }

    fn collapse_mask(&mut self, idy: N, mask: N) {
        let len = self.psi.len();
        let psi = Arc::new(take(&mut self.psi));

        self.psi = (0..len)
            .into_par_iter()
            .map_init(
                || psi.clone(),
                move |psi, idx|
                    if (idx ^ idy) & mask != 0 {
                        C::zero()
                    } else {
                        psi[idx]
                    }
            ).collect();

        self.normalize();
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

        let n = p
            .par_iter()
            .map(|&p| {
                let rnd: R = rand::thread_rng().sample(rand_distr::StandardNormal);
                p.sqrt() * rnd
            })
            .collect::<Vec<R>>();

        let n_sum = n.par_iter().sum::<R>();

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
        self.psi[0..8].fmt(f)
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