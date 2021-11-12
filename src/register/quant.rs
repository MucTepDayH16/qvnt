use {rand::prelude::*, rand_distr};
#[cfg(feature = "cpu")]
use rayon::prelude::*;
use std::{fmt, ops::{Mul, MulAssign}};
use crate::math::*;
use crate::prelude::quant::threading::Model;

const MIN_QREG_LEN: usize = 8;

mod threading {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum Model {
        Single,
        #[cfg(feature = "cpu")]
        Multi(N),
    }

    pub use Model::*;

    impl Model {
        pub fn and(self, other: Self) -> Self {
            match (self, other) {
                (Single, Single) => Single,
                #[cfg(feature = "cpu")]
                (Single, Multi(n)) => Multi(n),
                #[cfg(feature = "cpu")]
                (Multi(n), Single) => Multi(n),
                #[cfg(feature = "cpu")]
                (Multi(n), Multi(m)) => Multi(n.max(m)),
            }
        }
    }
}

/// [`Quantum register`](Reg)
///
/// __The heart of [`QVNT`](crate) crate.__ It represents a set of entangle qubits,
/// their collective wavefunction and techniques for controlling quantum state.
///
/// Theoretically it could contain up to 64 qubits, however it would require more than 2_000_000 Terabytes of RAM.
/// For a common computer with 4Gb RAM the limitation is 26 qubits.
/// If you have more than 4Gb just remember, that 1 additional qubit require twice more RAM.
/// More precise formula is:
///
/// ```MAX_QUBIT_COUNT = 24 + log2(MEM_CAPACITY_IN_GB)```
///
/// For practice purposes it will be enough.
///
/// To create quantum computer state, e.g. with 10 qubits, you can use this code:
///
/// ```rust
/// use qvnt::prelude::*;
///
/// let q = QReg::new(10);
/// ```
///
/// The quantum register ```q``` starts with state |0>.
/// To vary initial state of register, you may use [`init_state`](Reg::init_state) modifier:
///
/// ```rust
/// # use qvnt::prelude::*;
/// // it will create quantum register in state |123>
/// let q = QReg::new(10).init_state(123);
/// ```
///
/// After creation of quantum computer you would like to be able to control its state.
/// QVNT provide [`op`](crate::operator) module, which contains an amount of quantum gates.
/// [`QReg::apply()`](Reg::apply) method is to apply <sup>sorry for tautology :)</sup> them:
///
/// ```rust
/// # use qvnt::prelude::*;
/// // quantum gates change state, so register must be mutable
/// let mut q = QReg::new(2);
///
/// // controlled gates have to be unwrapped
/// let gate =  op::h(0b01) * op::x(0b10).c(0b01).unwrap();
/// q.apply(&gate);
/// ```
///
/// This is the example of entangled state, which means that if we will apply gate on or measure
/// first(second) qubit, second(first) will change it state automatically.
/// To show that, we will use [`get_probabilities`](Reg::get_probabilities()) method.
/// It could show us the probabilities of each state in quantum register:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let mut q = QReg::new(2);
/// # let gate = op::h(0b01) * op::x(0b10).c(0b01).unwrap();
/// q.apply(&gate);
/// let prob: Vec<f64> = q.get_probabilities();
/// println!("{:?}", prob);
/// # assert_eq!(prob, [0.5, 0.0, 0.0, 0.5]);
/// ```
///
/// Output will be ```[0.5, 0.0, 0.0, 0.5]```, which means that quantum state consists only of states |00> and |11>.
/// Thus, measuring first qubit (```|_0>``` or ```|_1>``` will always collapse second qubit to the same value.
/// So, this example is just a complicated version if *flipping a coin* example.
///
///
#[derive(Clone)]
pub struct Reg {
    th: threading::Model,
    psi: Vec<C>,
    q_num: N,
    q_mask: N,
}

impl Reg {
    pub fn new(q_num: N) -> Self {
        let q_size = 1_usize << q_num;

        let mut psi = vec![C_ZERO; q_size.max(MIN_QREG_LEN)];
        psi[0] = C_ONE;

        Self {
            th: threading::Single, psi, q_num,
            q_mask: q_size.wrapping_add(!0_usize),
        }
    }

    #[cfg(feature = "cpu")]
    pub fn num_threads(self, num_threads: usize) -> Option<Self> {
        if num_threads > rayon::current_num_threads() {
            None
        } else {
            Some(Self{ th: threading::Multi(num_threads), ..self })
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
        super::VReg::new_with_mask(self.q_mask)
    }

    pub fn get_vreg_by(&self, mask: N) -> Option<super::VReg> {
        if mask & !self.q_mask != 0 {
            None
        } else {
            Some(super::VReg::new_with_mask(mask))
        }
    }

    // TODO: add tests for combine
    pub (crate) fn combine(q: (&Self, &Self), c: M1) -> Option<Self> {
        if q.0.q_num == q.1.q_num {
            let mut q_reg = Self::new(q.0.q_num + 1);
            let q_mask = q.0.q_mask;

            match q.0.th.and(q.1.th) {
                Model::Single => {
                    q_reg.psi.iter_mut()
                        .enumerate()
                        .for_each(|(idx, v)| {
                            let q = (q.0.psi[q_mask & idx], q.1.psi[q_mask & idx]);
                            if !q_mask & idx == 0 {
                                *v = c[0b00] * q.0 + c[0b01] * q.1;
                            } else {
                                *v = c[0b10] * q.0 + c[0b11] * q.1;
                            }
                        });
                }
                #[cfg(feature = "cpu")]
                Model::Multi(n) => {
                    crate::threads::global_install(n, || {
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
                    })
                }
            }
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
        let th = self.th.and(other.th);

        let shift = (0u8, self.q_num as u8);
        let mask = (self.q_mask, other.q_mask);

        let q_num = self.q_num + other.q_num;
        let q_size = 1_usize << q_num;

        let psi = match th {
            Model::Single => {
                (0..q_size.max(MIN_QREG_LEN))
                    .into_iter()
                    .map(
                        move |idx| if idx < q_size {
                            self.psi[(idx >> shift.0) & mask.0] * other.psi[(idx >> shift.1) & mask.1]
                        } else {
                            C_ZERO
                        }
                    ).collect()
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    (0..q_size.max(MIN_QREG_LEN)).into_par_iter().map(
                        move |idx| if idx < q_size {
                            self.psi[(idx >> shift.0) & mask.0] * other.psi[(idx >> shift.1) & mask.1]
                        } else {
                            C_ZERO
                        }
                    ).collect()
                })
            }
        };

        Self {
            th, psi, q_num, q_mask: q_size.wrapping_add(!0_usize),
        }
    }

    pub fn apply<Op>(&mut self, op: &Op)
    where Op: crate::operator::applicable::Applicable {
        self.psi = op.apply(std::mem::take(&mut self.psi))
    }

    #[cfg(feature = "cpu")]
    pub fn apply_sync<Op>(&mut self, op: &Op)
    where Op: crate::operator::applicable::ApplicableSync {
        match self.th {
            Model::Single => self.apply(op),
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    self.psi = op.apply_sync(std::mem::take(&mut self.psi));
                })
            }
        }
    }

    fn normalize(&mut self) -> &mut Self {
        let norm = 1. / self.get_absolute().sqrt();
        match self.th {
            Model::Single => {
                self.psi.iter_mut().for_each(|v| *v *= norm)
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    self.psi.par_iter_mut().for_each(|v| *v *= norm)
                })
            }
        };
        self
    }

    pub fn get_polar(&self) -> Vec<(R, R)> {
        match self.th {
            Model::Single => {
                self.psi[..(1 << self.q_num)].iter()
                    .map(|z| z.to_polar()).collect()
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    self.psi[..(1 << self.q_num)].par_iter()
                        .map(|z| z.to_polar()).collect()
                })
            }
        }
    }

    pub fn get_probabilities(&self) -> Vec<R> {
        match self.th {
            Model::Single => {
                let abs: R = self.psi.iter()
                    .map(|z| z.norm_sqr())
                    .sum();
                self.psi[..(1 << self.q_num)].iter()
                    .map(|z| z.norm_sqr() / abs)
                    .collect()
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    let abs: R = self.psi.par_iter()
                        .map(|z| z.norm_sqr())
                        .sum();
                    self.psi[..(1 << self.q_num)].par_iter()
                        .map(|z| z.norm_sqr() / abs)
                        .collect()
                })
            }
        }
    }
    pub fn get_absolute(&self) -> R {
        match self.th {
            Model::Single => {
                self.psi.iter()
                    .map(|z| z.norm_sqr())
                    .sum()
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    self.psi.par_iter()
                        .map(|z| z.norm_sqr())
                        .sum()
                })
            }
        }
    }

    fn collapse_mask(&mut self, idy: N, mask: N) {
        match self.th {
            Model::Single => {
                self.psi.iter_mut()
                    .enumerate()
                    .for_each(
                        |(idx, psi)| {
                            if (idx ^ idy) & mask != 0 {
                                *psi = C_ZERO;
                            }
                        });
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    self.psi.par_iter_mut()
                        .enumerate()
                        .for_each(
                            |(idx, psi)| {
                                if (idx ^ idy) & mask != 0 {
                                    *psi = C_ZERO;
                                }
                            });
                })
            }
        }
    }
    pub fn measure_mask(&mut self, mask: N) -> super::CReg {
        let mask = mask & self.q_mask;
        if mask == 0 { return super::CReg::new(self.q_num); }

        let rand_idx = thread_rng().sample(
            rand_distr::WeightedIndex::new(
                self.get_probabilities()
            ).unwrap()
        );

        self.collapse_mask(rand_idx, mask);
        super::CReg::new(self.q_num).init_state(rand_idx & mask)
    }
    pub fn measure(&mut self) -> super::CReg {
        self.measure_mask(self.q_mask)
    }

    #[cfg(feature = "cpu")]
    pub fn sample_all(&self, count: N) -> Vec<N> {
        use std::cmp::Ordering;

        let p = self.get_probabilities();
        let c = count as R;
        let c_sqrt = c.sqrt();

        let (mut n, delta) = match self.th {
            Model::Single => {
                let mut rng = rand::thread_rng();
                let n = p.iter()
                    .map(|&p| {
                        let rnd: R = rng.sample(rand_distr::StandardNormal);
                        p.sqrt() * rnd
                    })
                    .collect::<Vec<R>>();

                let n_sum = n.iter().sum::<R>();

                let n = (0..self.psi.len())
                    .into_iter()
                    .map(|idx| {
                        ((c * p[idx] + c_sqrt * (n[idx] - n_sum * p[idx])).round() as Z).max(0) as N
                    })
                    .collect::<Vec<N>>();

                let delta = n.iter().sum::<N>() as Z - count as Z;

                (n, delta)
            }
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
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
                })
            }
        };
        match delta.cmp(&0) {
            Ordering::Less => {
                let delta = delta.abs() as N;
                let delta = (delta >> self.q_num, delta % self.q_mask);
                for (idx, n) in n.iter_mut().enumerate() {
                    *n += delta.0;
                    if idx < delta.1 {
                        *n += 1;
                    }
                }
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