pub use multi::MultiOp;

pub (crate) mod applicable;

pub (in crate::operator) mod atomic;
pub (in crate::operator) mod single;
pub (in crate::operator) mod multi;

pub mod op {
    use crate::math::{C, FRAC_PI_2, N, R};

    use super::{multi::*, single::*};

    /// Identity operator.
    ///
    /// For any quantum state |q>, identity operator does not change state.
    ///
    /// I |q> = |q>
    #[inline(always)]
    pub fn id() -> MultiOp {
        MultiOp::default()
    }

    /// Pauli *X* operator, *aka* NOT gate.
    ///
    /// Performs negation for given qubits.
    ///
    /// X |0> = |1>
    ///
    /// X |1> = |0>
    #[inline(always)]
    pub fn x(a_mask: N) -> MultiOp {
        pauli::x(a_mask).into()
    }
    /// *X* rotation operator.
    ///
    /// Performs *phase* radians rotation around X axis on a Bloch sphere.
    #[inline(always)]
    pub fn rx(phase: R, a_mask: N) -> MultiOp {
        rotate::rx(a_mask, phase).expect("Mask should contain 1 bit!").into()
    }
    /// *Ising XX* coupling gate.
    ///
    /// Performs *phase* radians rotation around XX axis on 2-qubit Bloch spheres.
    #[inline(always)]
    pub fn rxx(phase: R, ab_mask: N) -> MultiOp {
        rotate::rxx(ab_mask, phase).expect("Mask should contain 2 bit!").into()
    }

    #[inline(always)]
    pub fn y(a_mask: N) -> MultiOp {
        pauli::y(a_mask).into()
    }
    /// *Y* rotation operator.
    ///
    /// Performs *phase* radians rotation around Y axis on a Bloch sphere.
    #[inline(always)]
    pub fn ry(phase: R, a_mask: N) -> MultiOp {
        rotate::ry(a_mask, phase).expect("Mask should contain 1 bit!").into()
    }
    /// *Ising YY* coupling gate.
    ///
    /// Performs *phase* radians rotation around YY axis on 2-qubit Bloch spheres.
    #[inline(always)]
    pub fn ryy(phase: R, ab_mask: N) -> MultiOp {
        rotate::ryy(ab_mask, phase).expect("Mask should contain 2 bit!").into()
    }

    /// Pauli *Z* operator.
    ///
    /// Negate an amplitude of 1-state.
    ///
    /// Z |0> = |0>
    ///
    /// Z |1> = -|1>
    #[inline(always)]
    pub fn z(a_mask: N) -> MultiOp {
        pauli::z(a_mask).into()
    }
    /// *S* operator.
    ///
    /// Square root of *Z* operator.
    ///
    /// S |0> = |0>
    ///
    /// S |1> = i|1>
    #[inline(always)]
    pub fn s(a_mask: N) -> MultiOp {
        pauli::s(a_mask).into()
    }
    /// *S* operator.
    ///
    /// Fourth root of *Z* operator.
    ///
    /// T |0> = |0>
    ///
    /// T |1> = (1+i)/sqrt(2) |1>
    #[inline(always)]
    pub fn t(a_mask: N) -> MultiOp {
        pauli::t(a_mask).into()
    }
    /// *Z* rotation operator.
    ///
    /// Performs *phase* radians rotation around Z axis on a Bloch sphere.
    #[inline(always)]
    pub fn rz(phase: R, a_mask: N) -> MultiOp {
        rotate::rz(a_mask, phase).expect("Mask should contain 1 bit!").into()
    }
    /// *Ising ZZ* coupling gate.
    ///
    /// Performs *phase* radians rotation around ZZ axis on 2-qubit Bloch spheres.
    #[inline(always)]
    pub fn rzz(phase: R, ab_mask: N) -> MultiOp {
        rotate::rzz(ab_mask, phase).expect("Mask should contain 2 bit!").into()
    }

    /// Phase shift operator.
    ///
    /// Performs phase shift for a range of given qubits by corresponding phase.
    ///
    /// ```rust
    /// use qvnt::prelude::*;
    /// use consts::PI;
    ///
    /// //  Take a third root of *Z* gate.
    /// let z_pow_a = op::phi(vec![(PI / 3., 0b1)]);
    /// //  Equivalent to Op::z(0b1).
    /// let z = op::phi(vec![(PI, 0b1)]);
    /// ```
    #[inline(always)]
    pub fn phi(phases: Vec<(R, N)>) -> MultiOp {
        pauli::phi(phases).into()
    }

    /// *SWAP* gate.
    ///
    /// Performs SWAP of 2 qubits' value.
    ///
    /// SWAP |ab> = |ba>
    #[inline(always)]
    pub fn swap(ab_mask: N) -> MultiOp {
        swap::swap(ab_mask).expect("Mask should contain 2 bit!").into()
    }
    /// Square root of *SWAP* gate.
    ///
    /// Performs a *"half"* SWAP of 2 qubits' value.
    /// This gate could couple qubits.
    ///
    /// sqrt(SWAP) * sqrt(SWAP) |ab> = |ba>
    ///
    /// ```rust
    /// use qvnt::prelude::*;
    /// use consts::{_0, _1, _i};
    ///
    /// //  sqrt(SWAP) gate's matrix representation:
    /// assert_eq!(
    ///     op::sqrt_swap(0b11).matrix(2),
    ///     [   [_1, _0,              _0,              _0],
    ///         [_0, 0.5 * (_1 + _i), 0.5 * (_1 - _i), _0],
    ///         [_0, 0.5 * (_1 - _i), 0.5 * (_1 + _i), _0],
    ///         [_0, _0,              _0,              _1]]
    /// );
    /// ```
    #[inline(always)]
    pub fn sqrt_swap(ab_mask: N) -> MultiOp {
        swap::sqrt_swap(ab_mask).expect("Mask should contain 2 bit!").into()
    }
    /// *iSWAP* gate.
    ///
    /// Perform SWAP of 2 qubits' value, multiplying bu *i* if qubits are not equals.
    ///
    /// iSWAP |00> = |00>
    ///
    /// iSWAP |01> = i |01>
    ///
    /// iSWAP |10> = i |10>
    ///
    /// iSWAP |11> = |11>
    #[inline(always)]
    pub fn i_swap(ab_mask: N) -> MultiOp {
        swap::i_swap(ab_mask).expect("Mask should contain 2 bit!").into()
    }
    /// Square root of *iSWAP* gate.
    ///
    /// Performs a *"half"* iSWAP of 2 qubits' value.
    /// This gate could couple qubits.
    ///
    /// sqrt(iSWAP) * sqrt(iSWAP) |ab> = iSWAP |ab>
    ///
    /// ```rust
    /// use qvnt::prelude::*;
    /// use consts::{_0, _1, _i, SQRT_1_2};
    ///
    /// //  sqrt(iSWAP) gate's matrix representation:
    /// assert_eq!(
    ///     op::sqrt_i_swap(0b11).matrix(2),
    ///     [   [_1, _0,            _0,            _0],
    ///         [_0, SQRT_1_2 * _1, SQRT_1_2 * _i, _0],
    ///         [_0, SQRT_1_2 * _i, SQRT_1_2 * _1, _0],
    ///         [_0, _0,            _0,            _1]]
    /// );
    /// ```
    #[inline(always)]
    pub fn sqrt_i_swap(ab_mask: N) -> MultiOp {
        swap::sqrt_i_swap(ab_mask).expect("Mask should contain 2 bit!").into()
    }

    /// Hadamard gate.
    ///
    /// Performs Hadamard transform on a given qubits.
    /// This is the simplest operation that create a superposition from a pure state |i>
    ///
    /// H |0> = |+> = ( |0> + |1> ) / sqrt(2)
    ///
    /// H |1> = |-> = ( |0> - |1> ) / sqrt(2)
    #[inline(always)]
    pub fn h(a_mask: N) -> MultiOp {
        h::h(a_mask)
    }

    /// *U1(lam)* gate.
    ///
    /// First universal operator. Equivalent to *RZ* and U3(0,0,lam).
    #[inline(always)]
    pub fn u1(lam: R, a_mask: N) -> MultiOp {
        rz(lam, a_mask)
    }
    /// *U2(phi,lam)* gate.
    ///
    /// Second universal operator. Equivalent to U3(PI/2, phi, lam)
    #[inline(always)]
    pub fn u2(phi: R, lam: R, a_mask: N) -> MultiOp {
        rz(lam, a_mask) * ry(FRAC_PI_2, a_mask) * rz(phi, a_mask)
    }
    /// *U3(the,phi,lam)* gate.
    ///
    /// Third universal operator.
    ///
    /// 3 parameters are enough to describe any unitary operator.
    /// All gates could be expressed in term of *U3* up to a phase factor.
    ///
    /// X = i U3(PI,PI,0)
    ///
    /// Y = i U3(PI,0,0)
    ///
    /// Z = i U3(0,0,PI)
    ///
    /// Z^a = i U3(0,0,PI*a)
    #[inline(always)]
    pub fn u3(the: R, phi: R, lam: R, a_mask: N) -> MultiOp {
        rz(lam, a_mask) * ry(the, a_mask) * rz(phi, a_mask)
    }

    /// Discrete Fourier transform for the quantum state's amplitudes.
    ///
    /// Fourier transform with factor 1/sqrt(N).
    /// This transform keeps the norm of vector, so it could be applied as unitary operator.
    /// It use the technique of fast fourier transform and have O(n*log(n)) time complexity.
    ///
    /// Fourier transform on a single qubit is just a Hadamard gate.
    #[inline(always)]
    pub fn qft(a_mask: N) -> MultiOp {
        qft::qft(a_mask)
    }
    /// Discrete Fourier transform with qubits' swap
    ///
    /// QFT is differ from real DFT by a bit order of amplitudes indices.
    /// *qft_swapped* is a natural version of DFT.
    #[inline(always)]
    pub fn qft_swapped(a_mask: N) -> MultiOp {
        qft::qft_swapped(a_mask)
    }


    #[cfg(test)]
    pub(crate) fn bench_circuit() -> MultiOp {
        MultiOp::default()
            * h(0b111)
            * h(0b100).c(0b001).unwrap()
            * x(0b001).c(0b110).unwrap()
            * rx(1.2, 0b100)
            * phi(vec![ (1.0, 0b010) ]).c(0b001).unwrap()
            * h(0b001).c(0b100).unwrap()
            * z(0b010)
            * rxx(crate::math::FRAC_PI_6, 0b101)
    }
}
