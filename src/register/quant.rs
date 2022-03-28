use crate::math::*;
use crate::prelude::quant::threading::Model;
#[cfg(feature = "cpu")]
use rayon::prelude::*;
use std::{
    fmt,
    ops::{Mul, MulAssign},
};
use {rand::prelude::*, rand_distr};

const MIN_BUFFER_LEN: usize = 8;
const MAX_LEN_TO_DISPLAY: usize = 8;

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
#[derive(Clone)]
pub struct Reg {
    th: threading::Model,
    psi: Vec<C>,
    q_num: N,
    q_mask: N,
}

impl Reg {
    /// Create quantum register with a given number of bits.
    /// Initial value will be 0.
    pub fn new(q_num: N) -> Self {
        let q_size = 1_usize << q_num;

        let mut psi = vec![C_ZERO; q_size.max(MIN_BUFFER_LEN)];
        psi[0] = C_ONE;

        Self {
            th: threading::Single,
            psi,
            q_num,
            q_mask: q_size.wrapping_sub(1_usize),
        }
    }

    /// __This method available with "cpu" feature enabled.__
    ///
    /// Set specified number of threads for a given quantum register.
    /// This value is used all across other methods to accelerate execution, using threads of your computer.
    #[cfg(feature = "cpu")]
    pub fn num_threads(self, num_threads: usize) -> Option<Self> {
        if 0 == num_threads || num_threads > rayon::current_num_threads() {
            None
        } else if num_threads == 1 {
            Some(Self {
                th: threading::Single,
                ..self
            })
        } else {
            Some(Self {
                th: threading::Multi(num_threads),
                ..self
            })
        }
    }

    pub(crate) fn reset(&mut self, i_state: N) {
        self.psi = vec![C_ZERO; self.psi.len()];
        self.psi[self.q_mask & i_state] = C_ONE;
    }
    
    pub (crate) fn reset_by_mask(&mut self, mask: N) {
        match self.th {
            Model::Single => {
                self.psi.iter_mut().enumerate()
                    .filter(|(idx, _)| idx & mask != 0)
                    .for_each(|(_, psi)| *psi = C_ZERO);
            },
            #[cfg(feature = "cpu")]
            Model::Multi(n) => {
                crate::threads::global_install(n, || {
                    self.psi.par_iter_mut().enumerate()
                        .filter(|(idx, _)| idx & mask != 0)
                        .for_each(|(_, psi)| *psi = C_ZERO);
                })
            }
        }
    }

    /// Initialize state of qubits.
    pub fn init_state(mut self, i_state: N) -> Self {
        self.reset(i_state);
        self
    }

    /// Acquire the [`VReg`](super::VReg) for a whole quantum register.
    pub fn get_vreg(&self) -> super::VReg {
        super::VReg::new_with_mask(self.q_mask)
    }

    /// Acquire the [`VReg`](super::VReg) for a specified part of quantum register.
    pub fn get_vreg_by(&self, mask: N) -> Option<super::VReg> {
        if mask & !self.q_mask != 0 {
            None
        } else {
            Some(super::VReg::new_with_mask(mask))
        }
    }

    pub(crate) fn combine(q: (&Self, &Self)) -> Option<Self> {
        if q.0.q_num == q.1.q_num {
            let mut q_reg = Self::new(q.0.q_num + 1);

            match q.0.th {
                Model::Single => {
                    q_reg.psi[..q.0.psi.len()].clone_from_slice(&q.0.psi[..]);
                    q_reg.psi[q.0.psi.len()..].clone_from_slice(&q.1.psi[..]);
                }
                #[cfg(feature = "cpu")]
                Model::Multi(n) => crate::threads::global_install(n, || {
                    q_reg.psi[..q.0.psi.len()]
                        .par_iter_mut()
                        .zip(q.0.psi.par_iter())
                        .for_each(|p| *p.0 = *p.1);
                    q_reg.psi[q.0.psi.len()..]
                        .par_iter_mut()
                        .zip(q.1.psi.par_iter())
                        .for_each(|p| *p.0 = *p.1);
                }),
            }

            Some(q_reg)
        } else {
            None
        }
    }

    // TODO: add tests for combine
    pub(crate) fn combine_with_unitary(q: (&Self, &Self), c: M1) -> Option<Self> {
        #[cfg(feature = "float-cmp")]
        assert!(crate::math::matrix::is_unitary_m1(&c));
        if q.0.q_num == q.1.q_num {
            let mut q_reg = Self::new(q.0.q_num + 1);
            let q_mask = q.0.q_mask;

            match q.0.th.and(q.1.th) {
                Model::Single => {
                    q_reg.psi.iter_mut().enumerate().for_each(|(idx, v)| {
                        let q = (q.0.psi[q_mask & idx], q.1.psi[q_mask & idx]);
                        if !q_mask & idx == 0 {
                            *v = c[0b00] * q.0 + c[0b01] * q.1;
                        } else {
                            *v = c[0b10] * q.0 + c[0b11] * q.1;
                        }
                    });
                }
                #[cfg(feature = "cpu")]
                Model::Multi(n) => crate::threads::global_install(n, || {
                    q_reg.psi.par_iter_mut().enumerate().for_each(|(idx, v)| {
                        let q = (q.0.psi[q_mask & idx], q.1.psi[q_mask & idx]);
                        if !q_mask & idx == 0 {
                            *v = c[0b00] * q.0 + c[0b01] * q.1;
                        } else {
                            *v = c[0b10] * q.0 + c[0b11] * q.1;
                        }
                    });
                }),
            }
            Some(q_reg)
        } else {
            None
        }
    }

    // TODO: add tests for linear_composition
    pub(crate) fn linear_composition(&mut self, psi: &[C], c: (C, C)) {
        assert_eq!(self.psi.len(), psi.len());

        match self.th {
            Model::Single => self
                .psi
                .iter_mut()
                .zip(psi.iter())
                .for_each(|q| *q.0 = q.0.mul(c.0) + q.1.mul(c.1)),
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                self.psi
                    .par_iter_mut()
                    .zip(psi.par_iter())
                    .for_each(|q| *q.0 = q.0.mul(c.0) + q.1.mul(c.1))
            }),
        }
    }

    fn tensor_prod(self, other: Self) -> Self {
        let th = self.th.and(other.th);

        let shift = (0u8, self.q_num as u8);
        let mask = (self.q_mask, other.q_mask);

        let q_num = self.q_num + other.q_num;
        let q_size = 1_usize << q_num;

        let psi = match th {
            Model::Single => (0..q_size.max(MIN_BUFFER_LEN))
                .into_iter()
                .map(move |idx| {
                    if idx < q_size {
                        self.psi[(idx >> shift.0) & mask.0] * other.psi[(idx >> shift.1) & mask.1]
                    } else {
                        C_ZERO
                    }
                })
                .collect(),
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                (0..q_size.max(MIN_BUFFER_LEN))
                    .into_par_iter()
                    .map(move |idx| {
                        if idx < q_size {
                            self.psi[(idx >> shift.0) & mask.0]
                                * other.psi[(idx >> shift.1) & mask.1]
                        } else {
                            C_ZERO
                        }
                    })
                    .collect()
            }),
        };

        Self {
            th,
            psi,
            q_num,
            q_mask: q_size.wrapping_sub(1_usize),
        }
    }

    /// Apply quantum gate to register.
    /// This method only works in single threading model.
    /// To accelerate it you may use [`apply_sync`].
    pub fn apply<Op>(&mut self, op: &Op)
    where
        Op: crate::operator::applicable::Applicable,
    {
        match self.th {
            Model::Single => {
                let mut psi = Vec::with_capacity(self.psi.capacity());
                unsafe { psi.set_len(self.psi.len()) };
                op.apply(&self.psi, &mut psi);
                std::mem::swap(&mut self.psi, &mut psi);
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                let mut psi = Vec::with_capacity(self.psi.capacity());
                unsafe { psi.set_len(self.psi.len()) };
                op.apply_sync(&self.psi, &mut psi);
                std::mem::swap(&mut self.psi, &mut psi);
            }),
        }
    }

    /// __This method available with "cpu" feature enabled.__
    ///
    /// Apply quantum gate to register, using specified number of threads in [`num_threads`](Reg::num_threads).
    #[deprecated(since = "0.3.3", note = "use `apply` instead")]
    #[cfg(feature = "cpu")]
    pub fn apply_sync<Op>(&mut self, op: &Op)
    where
        Op: crate::operator::applicable::Applicable,
    {
        match self.th {
            Model::Single => self.apply(op),
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                let mut psi = Vec::with_capacity(self.psi.capacity());
                unsafe { psi.set_len(self.psi.len()) };
                op.apply_sync(&self.psi, &mut psi);
                std::mem::swap(&mut self.psi, &mut psi);
            }),
        }
    }

    fn normalize(&mut self) -> &mut Self {
        let norm = 1. / self.get_absolute().sqrt();
        match self.th {
            Model::Single => self.psi.iter_mut().for_each(|v| *v *= norm),
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                self.psi.par_iter_mut().for_each(|v| *v *= norm)
            }),
        };
        self
    }

    /// Return complex amplitudes of quantum states of register in polar form.
    pub fn get_polar(&self) -> Vec<(R, R)> {
        match self.th {
            Model::Single => self.psi[..(1 << self.q_num)]
                .iter()
                .map(|z| z.to_polar())
                .collect(),
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                self.psi[..(1 << self.q_num)]
                    .par_iter()
                    .map(|z| z.to_polar())
                    .collect()
            }),
        }
    }

    /// Return probabilities of quantum states of register.
    pub fn get_probabilities(&self) -> Vec<R> {
        match self.th {
            Model::Single => {
                let abs: R = self.psi.iter().map(|z| z.norm_sqr()).sum();
                self.psi[..(1 << self.q_num)]
                    .iter()
                    .map(|z| z.norm_sqr() / abs)
                    .collect()
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                let abs: R = self.psi.par_iter().map(|z| z.norm_sqr()).sum();
                self.psi[..(1 << self.q_num)]
                    .par_iter()
                    .map(|z| z.norm_sqr() / abs)
                    .collect()
            }),
        }
    }

    /// Return absolute value of wavefunction of quantum register.
    /// If you use gates from [`op`](crate::operator) module, it always will be 1.
    pub fn get_absolute(&self) -> R {
        match self.th {
            Model::Single => self.psi.iter().map(|z| z.norm_sqr()).sum(),
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                self.psi.par_iter().map(|z| z.norm_sqr()).sum()
            }),
        }
    }

    fn collapse_mask(&mut self, idy: N, mask: N) {
        match self.th {
            Model::Single => {
                self.psi.iter_mut().enumerate().for_each(|(idx, psi)| {
                    if (idx ^ idy) & mask != 0 {
                        *psi = C_ZERO;
                    }
                });
            }
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
                self.psi.par_iter_mut().enumerate().for_each(|(idx, psi)| {
                    if (idx ^ idy) & mask != 0 {
                        *psi = C_ZERO;
                    }
                });
            }),
        }
    }

    /// Measure specified qubits into classical register.
    /// Wavefunction of quantum register will collapse after measurement.
    pub fn measure_mask(&mut self, mask: N) -> super::CReg {
        let mask = mask & self.q_mask;
        if mask == 0 {
            return super::CReg::new(self.q_num);
        }

        let rand_idx =
            thread_rng().sample(rand_distr::WeightedIndex::new(self.get_probabilities()).unwrap());

        self.collapse_mask(rand_idx, mask);
        super::CReg::new(self.q_num).init_state(rand_idx & mask)
    }

    /// Measure all qubits into classical register.
    /// Wavefunction of quantum register will collapse after measurement.
    pub fn measure(&mut self) -> super::CReg {
        self.measure_mask(self.q_mask)
    }

    /// Make a histogram for quantum register.
    /// This histogram also could be obtained by calling [`measure`](Reg::measure) *count* times.
    /// But [`sample_all`](Reg::sample_all) does not collapse wavefunction and executes __MUSH FASTER__.
    /// If you want to simulate the execution of quantum computer, you would prefer [`sample_all`](Reg::sample_all).
    pub fn sample_all(&self, count: N) -> Vec<N> {
        use std::cmp::Ordering;

        let p = self.get_probabilities();
        let c = count as R;
        let c_sqrt = c.sqrt();

        let (mut n, delta) = match self.th {
            Model::Single => {
                let mut rng = rand::thread_rng();
                let n = p
                    .iter()
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
            #[cfg(feature = "cpu")]
            Model::Multi(n) => crate::threads::global_install(n, || {
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
            }),
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
            }
            Ordering::Greater => {
                let mut delta = delta as N;
                for idx in 0.. {
                    if delta == 0 {
                        break;
                    }
                    if n[idx & self.q_mask] == 0 {
                        continue;
                    }
                    n[idx & self.q_mask] -= 1;
                    delta -= 1;
                }
            }
            _ => {}
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
        if 1 << self.q_num <= MAX_LEN_TO_DISPLAY {
            self.psi[..(1 << self.q_num)]
                .iter()
                .enumerate()
                .fold(&mut f.debug_struct("QReg"), |f, (idx, psi)| {
                    f.field(&format!("{}", idx), psi)
                })
                .finish()
        } else {
            self.psi[..MAX_LEN_TO_DISPLAY]
                .iter()
                .enumerate()
                .fold(&mut f.debug_struct("QReg"), |f, (idx, psi)| {
                    f.field(&format!("{}", idx), psi)
                })
                .finish_non_exhaustive()
        }
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn quantum_reg() {
        use crate::math::C;

        let mut reg = QReg::new(4).init_state(0b1100);
        let mask = 0b0110;

        let operator =
            op::h(0b1111) * op::h(0b0011).c(0b1000).unwrap() * op::swap(0b1001).c(0b0010).unwrap();

        reg.apply(&operator);

        assert_eq!(format!("{:?}", operator), "[H3, H12, C8_H3, C2_SWAP9]");
        assert_eq!(
            reg.psi,
            [
                C { re: 0.25, im: 0.0 },
                C { re: 0.25, im: 0.0 },
                C { re: 0.25, im: 0.0 },
                C { re: 0.0, im: 0.0 },
                C { re: -0.25, im: 0.0 },
                C { re: -0.25, im: 0.0 },
                C { re: -0.25, im: 0.0 },
                C { re: 0.0, im: 0.0 },
                C { re: -0.5, im: 0.0 },
                C { re: 0.0, im: 0.0 },
                C { re: 0.25, im: 0.0 },
                C { re: 0.0, im: 0.0 },
                C { re: 0.5, im: 0.0 },
                C { re: 0.0, im: 0.0 },
                C { re: -0.25, im: 0.0 },
                C { re: 0.0, im: 0.0 },
            ]
        );
        assert_eq!(format!("{:?}", reg), "QReg { 0: Complex { re: 0.25, im: 0.0 }, 1: Complex { re: 0.25, im: 0.0 }, 2: Complex { re: 0.25, im: 0.0 }, 3: Complex { re: 0.0, im: 0.0 }, 4: Complex { re: -0.25, im: 0.0 }, 5: Complex { re: -0.25, im: 0.0 }, 6: Complex { re: -0.25, im: 0.0 }, 7: Complex { re: 0.0, im: 0.0 }, .. }".to_string());

        assert_eq!(reg.measure_mask(mask).get() & !mask, 0);
    }

    #[test]
    fn tensor() {
        const EPS: f64 = 1e-9;

        let pend_ops = op::h(0b01);

        let mut reg1 = QReg::new(2).init_state(0b01);
        let mut reg2 = QReg::new(1).init_state(0b1);

        reg1.apply(&pend_ops);
        reg2.apply(&pend_ops);

        let test_prob = (reg1 * reg2).get_probabilities();
        let true_prob = vec![0.25, 0.25, 0., 0., 0.25, 0.25, 0., 0.];

        assert!(test_prob
            .into_iter()
            .zip(true_prob.into_iter())
            .all(|(a, b)| (a - b).abs() < EPS));

        let mut reg3 = QReg::new(3).init_state(0b101);
        let pend_ops = op::h(0b101);

        reg3.apply(&pend_ops);

        let test_prob = reg3.get_probabilities();
        let true_prob = vec![0.25, 0.25, 0., 0., 0.25, 0.25, 0., 0.];

        assert!(test_prob
            .into_iter()
            .zip(true_prob.into_iter())
            .all(|(a, b)| (a - b).abs() < EPS));
    }

    #[test]
    fn histogram() {
        let mut q = QReg::new(8).init_state(123);

        q.apply(&op::h(255));

        for _ in 0..10 {
            let hist = q.sample_all(2048);
            assert_eq!(hist.len(), 256);
            assert_eq!(hist.iter().sum::<usize>(), 2048);
        }
    }
}
