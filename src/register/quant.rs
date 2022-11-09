#![allow(clippy::uninit_vec)]

use std::{
    fmt,
    ops::{Mul, MulAssign},
};

use rand::prelude::*;
use rand_distr;

use crate::{
    backend::{Backend, BackendBuilder, DefaultBuilder},
    math::*,
    operator::applicable::Applicable,
};

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
/// ```rust
/// use qvnt::prelude::*;
///
/// let q = QReg::new(10);
/// ```
/// 
/// The quantum register ```q``` starts with state |0>.
/// To vary initial state of register, you may use [`with_state`](Reg::with_state) modifier:
/// ```rust
/// # use qvnt::prelude::*;
/// // it will create quantum register in state |123>
/// let q = QReg::with_state(10, 123);
/// ```
/// 
/// After creation of quantum computer you would like to be able to control its state.
/// QVNT provide [`op`](crate::operator) module, which contains an amount of quantum gates.
/// [`QReg::apply()`](Reg::apply) method is to apply <sup>sorry for tautology :)</sup> them:
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
pub struct Reg<B: Backend> {
    backend: B,
    q_num: N,
    q_mask: Mask,
}

impl Reg<<DefaultBuilder as BackendBuilder>::Backend> {
    /// Create quantum register with a given number of bits.
    /// Initial value will be set to 0.
    pub fn new(q_num: N) -> Self {
        Self::with_builder(q_num, DefaultBuilder::default())
    }

    /// Initialize state of qubits.
    pub fn with_state(q_num: N, state: Mask) -> Self {
        Self::with_state_and_builder(q_num, state, DefaultBuilder::default())
    }
}

impl<B: Backend> Reg<B> {
    pub fn with_builder(q_num: N, builder: impl BackendBuilder<Backend = B>) -> Self {
        Self::with_state_and_backend(q_num, 0, builder.build(q_num).unwrap())
    }

    pub fn with_state_and_builder(
        q_num: N,
        state: Mask,
        builder: impl BackendBuilder<Backend = B>,
    ) -> Self {
        Self::with_state_and_backend(q_num, state, builder.build(q_num).unwrap())
    }

    #[inline(always)]
    fn with_state_and_backend(q_num: N, state: Mask, mut backend: B) -> Self {
        backend.reset_state(state).unwrap();

        Self {
            backend,
            q_num,
            q_mask: (1_usize << q_num).wrapping_sub(1_usize),
        }
    }
}

impl<B: Backend> Reg<B> {
    pub fn num(&self) -> N {
        self.q_num
    }

    pub fn set_num(&mut self, q_num: N) {
        B::reset_state_and_size(&mut self.backend, q_num, 0).unwrap();
    }

    pub(crate) fn reset(&mut self, state: Mask) {
        let state = self.q_mask & state;
        B::reset_state(&mut self.backend, state).unwrap();
    }

    pub(crate) fn reset_by_mask(&mut self, mask: Mask) {
        self.collapse_mask(0, mask)
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

    pub(crate) fn combine(_q: (&Self, &Self)) -> Option<Self> {
        unimplemented!()
    }

    pub(crate) fn combine_with_unitary(_q: (&Self, &Self), _c: M1) -> Option<Self> {
        unimplemented!()
    }

    pub(crate) fn linear_composition(&mut self, _psi: &[C], _c: (C, C)) {
        unimplemented!()
    }

    fn tensor_prod_assign(&mut self, mut other: Self) {
        let other_psi = other.backend.drain();
        self.backend.tensor_prod_assign(other_psi).unwrap();
        self.q_num += other.q_num;
        self.q_mask = (1usize << self.q_num).saturating_sub(1);
    }

    #[inline(always)]
    fn tensor_prod(mut self, other: Self) -> Self {
        self.tensor_prod_assign(other);
        self
    }

    /// Apply quantum gate to register.
    /// This method only works in single threading model.
    pub fn apply<Op>(&mut self, op: &Op)
    where
        Op: Applicable,
    {
        op.apply(&mut self.backend).unwrap();
    }

    /// Return complex amplitudes of quantum states of register in polar form.
    pub fn get_polar(&self) -> Vec<(R, R)> {
        B::collect(&self.backend)
            .into_iter()
            .map(|c| c.to_polar())
            .collect()
    }

    /// Return probabilities of quantum states of register.
    pub fn get_probabilities(&self) -> Vec<R> {
        B::collect_probabilities(&self.backend)
    }

    /// Return absolute value of wavefunction of quantum register.
    /// If you use gates from [`op`](crate::operator) module, it always will be 1.
    pub fn get_absolute(&self) -> R {
        B::collect_probabilities(&self.backend).into_iter().sum()
    }

    fn collapse_mask(&mut self, collapse_state: Mask, mask: Mask) {
        if mask == self.q_mask {
            B::reset_state(&mut self.backend, 0)
        } else {
            B::collapse_by_mask(&mut self.backend, collapse_state, mask)
        }
        .unwrap()
    }

    /// Measure specified qubits into classical register.
    /// Wavefunction of quantum register will collapse after measurement.
    pub fn measure_mask(&mut self, mask: Mask) -> super::CReg {
        let mask = mask & self.q_mask;
        if mask == 0 {
            return super::CReg::new(self.q_num);
        }

        let rand_idx =
            thread_rng().sample(rand_distr::WeightedIndex::new(self.get_probabilities()).unwrap());

        self.collapse_mask(rand_idx, mask);
        super::CReg::with_state(self.q_num, rand_idx & mask)
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
        let p = self.get_probabilities();
        let c = count as R;
        let c_sqrt = c.sqrt();

        let mut rng = rand::thread_rng();
        let p_sqrt_distr = p
            .iter()
            .map(|&p| rng.sample(rand_distr::Normal::new(0.0, p.sqrt()).unwrap()))
            .collect::<Vec<R>>();
        let p_sqrt_distr_sum = p_sqrt_distr.iter().sum::<R>();

        let mut n = p
            .iter()
            .zip(&p_sqrt_distr)
            .map(|(p, p_sqrt_distr)| {
                ((c * p + c_sqrt * (p_sqrt_distr - p_sqrt_distr_sum * p)).round() as Z).max(0) as N
            })
            .collect::<Vec<N>>();
        let n_sum: N = n.iter().sum();

        if n_sum < count {
            let delta = count - n_sum;
            let (delta_for_each, extra_one_idx) = (delta >> self.q_num, delta % self.q_mask);
            for (idx, n) in n.iter_mut().enumerate() {
                *n += delta_for_each;
                if idx < extra_one_idx {
                    *n += 1;
                }
            }
        } else if n_sum > count {
            let mut delta = n_sum - count;
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

        n
    }
}

// impl<B: Backend> Default for Reg<B> {
//     fn default() -> Self {
//         Self::new(0)
//     }
// }

impl<B: Backend> fmt::Debug for Reg<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        B::fmt(&self.backend, f)
    }
}

impl<B: Backend> Mul for Reg<B> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.tensor_prod(other)
    }
}

impl<B: Backend> MulAssign for Reg<B> {
    fn mul_assign(&mut self, rhs: Self) {
        self.tensor_prod_assign(rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn quantum_reg() {
        use crate::math::C;

        let mut reg = QReg::with_state(4, 0b1100);
        let mask = 0b0110;

        let operator =
            op::h(0b1111) * op::h(0b0011).c(0b1000).unwrap() * op::swap(0b1001).c(0b0010).unwrap();

        reg.apply(&operator);

        assert_eq!(
            format!("{:?}", operator),
            "[Op { name: \"H3\" }, Op { name: \"H12\" }, Op { name: \"C8_H3\" }, Op { name: \"C2_SWAP9\" }]"
        );
        assert_eq!(
            reg.backend.psi_main,
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

        for _ in 0..10 {
            assert_eq!(reg.clone().measure_mask(mask).get() & !mask, 0);
        }
    }

    #[test]
    fn tensor() {
        const EPS: f64 = 1e-9;

        let pend_ops = op::h(0b01);

        let mut reg1 = QReg::with_state(2, 0b01);
        let mut reg2 = QReg::with_state(1, 0b1);

        reg1.apply(&pend_ops);
        reg2.apply(&pend_ops);

        let test_prob = (reg1 * reg2).get_probabilities();
        let true_prob = vec![0.25, 0.25, 0., 0., 0.25, 0.25, 0., 0.];

        assert_eq!(test_prob.len(), true_prob.len());
        assert!(test_prob
            .iter()
            .zip(true_prob)
            .all(|(a, b)| (a - b).abs() < EPS));

        let mut reg3 = QReg::with_state(3, 0b101);
        let pend_ops = op::h(0b101);

        reg3.apply(&pend_ops);

        let test_prob = reg3.get_probabilities();
        let true_prob = vec![0.25, 0.25, 0., 0., 0.25, 0.25, 0., 0.];

        assert_eq!(test_prob.len(), true_prob.len());
        assert!(test_prob
            .iter()
            .zip(true_prob)
            .all(|(a, b)| (a - b).abs() < EPS));
    }

    #[test]
    fn histogram() {
        let mut q = QReg::with_state(8, 123);

        q.apply(&op::h(255));

        for _ in 0..10 {
            let hist = q.sample_all(2048);
            assert_eq!(hist.len(), 256);
            assert_eq!(hist.iter().sum::<usize>(), 2048);
        }
    }
}
