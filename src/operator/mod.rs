//! Module for representing quantum operations (*aka* gates).
//!
//! QVNT provides common 1-qubit and 2-qubits gates for constructing your own quantum circuits.
//! Moreover, it provide an interface to manipulate gates to create *controlled* or *inverse*
//! quantum operations from existing ones.
//!
//! Quantum gates in QVNT are *lazy*. This means they are evaluated on demand
//! via [`QReg::apply`](crate::register::QReg::apply)
//! and do not allocate __LOT__ of memory (only gate's metadata).
//!
//! # Operations
//!
//! [`SingleOp`] and [`MultiOp`] treated by quantum register identically.
//! But there are two difference between them:
//! * [`MultiOp`] contains a queue of [`SingleOp`]'s, which are applied sequentially;
//! * [`SingleOp`] does not have public constructor, so the only usage of it is to be stored in [`MultiOp`].
//!
//! # [`Applicable`] trait
//!
//! Trait represents a generalized quantum operator.
//! It is used to create new gates, without collecting it from existing gates.
//!
//! The main restriction for gates in quantum computing is to be *linear*.
//! Usage of a non-linear gate leads to non-linear quantum system.
//! However, non-linear gates could be useful for simulating external interferences of qubits system.
//!
//! __QVNT does not provide functionality for checking linearity, so it is on your own risk.__
//!
//! # Usage
//!
//! QVNT provides full set of quantum gates from OpenQASM standard with a similar names.
//! Ones could be obtained via [`op`](self) alias.
//! The simplest *really* quantum gate is called Hadamard gate, which is similar to flipping a coin:
//!
//! ```rust
//! # use qvnt::prelude::*;
//! // Create Hadamard gate, which acts on first qubit
//! let hadamard = op::h(0b1,);
//! let mut coin = QReg::new(1,);
//!
//! // Flip a coin
//! coin.apply(&hadamard,);
//! ```
//!
//! Now we have register, which have equal probabilities for 0 and 1.
//! To see what you can do with register, you could visit [`QReg`](crate::register::QReg) page.
//!
//! Example for 8 qubit Hadamard gate:
//!
//! ```rust
//! # use qvnt::prelude::*;
//! // Create Hadamard gate, which acts on 1, 2, 4, 5, 6 and 8 qubits
//! // 123 = 0b01111011
//! let ops = op::h(123,);
//! // Create register with 3 qubit in state |1> and all others in state |0>
//! // 4    = 0b00000100
//! let mut reg = QReg::new(8,).init_state(4,);
//!
//! reg.apply(&ops,);
//! ```
//!
//! This gate could be represented as:
//!
//! ```ignore
//! (q0) |0> -- [H]
//! (q1) |0> -- [H]
//! (q2) |1> -- [ ]
//! (q3) |0> -- [H]
//! (q4) |0> -- [H]
//! (q5) |0> -- [H]
//! (q6) |0> -- [H]
//! (q7) |0> -- [ ]
//! ```
//!
//! Unfortunately, output could not be showed here, because *qubyte* (8 qubits) contains 2<sup>8</sup> = 256 complex values.
//! Also, Hadamard gate creates state with equal probabilities and measuring register leads to a random value
//! independent of initial state. So, adding this to previous code:
//!
//! ```rust
//! # use qvnt::prelude::*;
//! # let mut  reg = QReg::new(8).init_state(4);
//! let c = reg.measure();
//! println!("{}", c.get());
//! ```
//!
//! will print equally distributed values from 0 to 255, which have bit 3 equal to zero and 8 bit equal to one.
//!
//! # Gate's modifiers - [`.c(...)`](crate::prelude::Applicable::c) and [`.dgr()`](crate::prelude::Applicable::dgr)

pub use self::{applicable::*, multi::MultiOp, single::SingleOp};
use self::{multi::*, single::*};
use crate::math::{C, FRAC_PI_2, N, R};

pub(crate) mod applicable;

pub(self) mod atomic;
pub(self) mod multi;
pub(self) mod single;

/// [`Identity`](id) gate.
///
/// For any quantum state |q> identity operator does not change state.
///
/// ```Id |q> = |q>```
/// 
/// Matrix form for [`Identity`](id) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th></tr>
/// </table>
#[inline(always)]
pub fn id() -> MultiOp {
    MultiOp::default()
}

/// Pauli [`X`](x) gate, aka NOT gate.
///
/// Performs negation for given qubit.
///
/// ```X |0> = |1>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;```X |1> = |0>```
/// 
/// Matrix form for [`X`](x) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
/// </table>
///
/// [`X`](x) gate acts on Bloch sphere:
///
/// | arbitrary state | state 0 | state 1 |
/// | :---: | :---: | :---: |
/// | ![X](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/X.gif) | ![X0](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/X0.gif) |  ![X1](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/X1.gif) |
#[inline(always)]
pub fn x(a_mask: N,) -> MultiOp {
    pauli::x(a_mask,).into()
}

/// *X* rotation gate.
///
/// Performs ```phase``` radians rotation around X axis on a Bloch sphere.
///
/// Matrix form for [`RX(λ)`](rx) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;cos(λ/2)</th><th>- <i>i</i> sin(λ/2)</th></tr>
///     <tr><th>- <i>i</i> sin(λ/2)</th><th>&nbsp;cos(λ/2)</th></tr>
/// </table>
#[inline(always)]
pub fn rx(phase: R, a_mask: N,) -> MultiOp {
    rotate::rx(a_mask, phase,)
        .expect("Mask should contain 1 bit!",)
        .into()
}

/// *Ising XX* coupling gate.
///
/// Performs *phase* radians rotation around XX axis on 2-qubit Bloch spheres.
///
/// Matrix form for [`RXX(λ)`](rxx) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>cos(λ/2)</th><th>&nbsp;&nbsp;0</th><th>&nbsp;&nbsp;0</th><th>- <i>i</i> sin(λ/2)</th></tr>
///     <tr><th>&nbsp;&nbsp;0</th><th>cos(λ/2)</th><th>- <i>i</i> sin(λ/2)</th><th>&nbsp;&nbsp;0</th></tr>
///     <tr><th>&nbsp;&nbsp;0</th><th>- <i>i</i> sin(λ/2)</th><th>cos(λ/2)</th><th>&nbsp;&nbsp;0</th></tr>
///     <tr><th>- <i>i</i> sin(λ/2)</th><th>&nbsp;&nbsp;0</th><th>&nbsp;&nbsp;0</th><th>cos(λ/2)</th></tr>
/// </table>
#[inline(always)]
pub fn rxx(phase: R, ab_mask: N,) -> MultiOp {
    rotate::rxx(ab_mask, phase,)
        .expect("Mask should contain 2 bit!",)
        .into()
}

/// Pauli [`Y`](y) gate.
///
/// It's effect could be determined from equation ```Y = iXZ```.
/// So *Y* gate does this things:
/// * negate the aplitude of |1> state ([`Z`](z));
/// * negate the qubit state ([`X`](x));
/// * multiply the aplitude by *i*.
///
/// ```Y |0> = i|1>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;```Y |1> = -i|0>```
/// 
/// Matrix form for [`Y`](y) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;- <i>i</i>&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;<i>i</i>&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
/// </table>
///
/// [`Y`](y) gate acts on Bloch sphere:
///
/// | arbitrary state | state 0 | state 1 |
/// | :---: | :---: | :---: |
/// | ![Y](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/Y.gif) | ![Y0](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/Y0.gif) |  ![Y1](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/Y1.gif) |
#[inline(always)]
pub fn y(a_mask: N,) -> MultiOp {
    pauli::y(a_mask,).into()
}

/// *Y* rotation gate.
///
/// Performs *phase* radians rotation around Y axis on a Bloch sphere.
///
/// Matrix form for [`RY(λ)`](ry) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>cos(λ/2)</th><th>-sin(λ/2)</th></tr>
///     <tr><th>sin(λ/2)</th><th>cos(λ/2)</th></tr>
/// </table>
#[inline(always)]
pub fn ry(phase: R, a_mask: N,) -> MultiOp {
    rotate::ry(a_mask, phase,)
        .expect("Mask should contain 1 bit!",)
        .into()
}

/// *Ising YY* coupling gate.
///
/// Performs *phase* radians rotation around YY axis on 2-qubit Bloch spheres.
///
/// Matrix form for [`RYY(λ)`](ryy) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>cos(λ/2)</th><th>&nbsp;&nbsp;0</th><th>&nbsp;&nbsp;0</th><th><i>i</i> sin(λ/2)</th></tr>
///     <tr><th>&nbsp;&nbsp;0</th><th>cos(λ/2)</th><th>- <i>i</i> sin(λ/2)</th><th>&nbsp;&nbsp;0</th></tr>
///     <tr><th>&nbsp;&nbsp;0</th><th>- <i>i</i> sin(λ/2)</th><th>cos(λ/2)</th><th>&nbsp;&nbsp;0</th></tr>
///     <tr><th><i>i</i> sin(λ/2)</th><th>&nbsp;&nbsp;0</th><th>&nbsp;&nbsp;0</th><th>cos(λ/2)</th></tr>
/// </table>
#[inline(always)]
pub fn ryy(phase: R, ab_mask: N,) -> MultiOp {
    rotate::ryy(ab_mask, phase,)
        .expect("Mask should contain 2 bit!",)
        .into()
}

/// Pauli [`Z`](z) gate.
///
/// Negate an amplitude of |1> qubit state.
///
/// ```Z |0> = |0>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;```Z |1> = -|1>```
/// 
/// Matrix form for [`Z`](z) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;- 1&nbsp;</th></tr>
/// </table>
///
/// [`Z`](z) gate acts on Bloch sphere:
///
/// | arbitrary state | state 0 | state 1 |
/// | :---: | :---: | :---: |
/// | ![Z](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/Z.gif) | ![Z0](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/Z0.gif) |  ![Z1](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/Z1.gif) |
#[inline(always)]
pub fn z(a_mask: N,) -> MultiOp {
    pauli::z(a_mask,).into()
}

/// Phase [`S`](s) gate.
///
/// Square root of [`Z`](z) gate.
///
/// ```S |0> = |0>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```S |1> = i|1>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```S S |q> = Z |q>```
/// 
/// Matrix form for [`S`](s) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;<i>i</i></th></tr>
/// </table>
#[inline(always)]
pub fn s(a_mask: N,) -> MultiOp {
    pauli::s(a_mask,).into()
}

/// Phase [`T`](t) gate.
///
/// Fourth root of [`Z`](z) gate and square root of [`S`](s) gate.
///
/// ```T |0> = |0>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```T |1> = i|1>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```T T |q> = S |q>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```T T T T |q> = Z |q>```
///
/// Matrix form for [`T`](t) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>(1+<i>i</i>)/&radic;2</th></tr>
/// </table>
#[inline(always)]
pub fn t(a_mask: N,) -> MultiOp {
    pauli::t(a_mask,).into()
}

/// *Z* rotation gate.
///
/// Performs *phase* radians rotation around Z axis on a Bloch sphere.
///
/// There are representations for [`Z`](z), [`S`](s) and [`T`](t) gates using rotation gate:
///
/// ```RZ(π) |q> = Z |q>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```RZ(π/2) |q> = S |q>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```RZ(π/4) |q> = T |q>```
/// 
/// And more general one:
/// ```Z^a |q> = RZ(aπ) |q>```
///
/// Matrix form for [`RZ(λ)`](rz) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>e<sup> - <i>i</i>λ/2</sup></th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>e<sup> <i>i</i>λ/2</sup></th></tr>
/// </table>
#[inline(always)]
pub fn rz(phase: R, a_mask: N,) -> MultiOp {
    rotate::rz(a_mask, phase,)
        .expect("Mask should contain 1 bit!",)
        .into()
}

/// *Ising ZZ* coupling gate.
///
/// Performs *phase* radians rotation around ZZ axis on 2-qubit Bloch spheres.
///
/// Matrix form for [`RZZ(λ)`](rzz) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>e<sup> - <i>i</i>λ/2</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>e<sup> <i>i</i>λ/2</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>e<sup> <i>i</i>λ/2</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>e<sup> - <i>i</i>λ/2</th></tr>
/// </table>
#[inline(always)]
pub fn rzz(phase: R, ab_mask: N,) -> MultiOp {
    rotate::rzz(ab_mask, phase,)
        .expect("Mask should contain 2 bit!",)
        .into()
}

/// Phase shift gate.
///
/// Performs phase shift for a range of given qubits by corresponding phase.
/// It is similar to [`RZ(λ)`](rz) gate, but do not perform global phase shift and can act on multiple qubits at once.
///
/// As an argument, it takes [`Vec<(f64,usize)>`](Vec).
/// Each element ```x``` of the vector is treated as separate phase shift by ```x.0```,
/// that act on qubits by mask ```x.1```.
///
/// ```rust
/// # use qvnt::prelude::*;
/// use std::f64::consts::PI as π;
///
/// //  Take a third root of *Z* gate.
/// let z_pow_a = op::phi(vec![(π / 3., 0b1,)],);
/// //  Equivalent to Op::z(0b1).
/// let z = op::phi(vec![(π, 0b1,)],);
/// ```
///
/// Its matrix form depend on [`Vec`] size, but for ```vec![(a, 1)]```, which affect only 1 qubit, the matrix is:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</sup></th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>e<sup> <i>i</i>a</sup></th></tr>
/// </table>
#[deprecated(note = "it is overhead, use chain of `rz` instead")]
#[allow(deprecated)]
#[inline(always)]
pub fn phi(phases: Vec<(R, N,),>,) -> MultiOp {
    pauli::phi(phases,).into()
}

/// [`SWAP`](swap()) gate.
///
/// Performs SWAP of 2 qubits' state.
///
/// ```SWAP |00> = |00>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```SWAP |01> = |10>```
///
/// ```SWAP |10> = |01>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```SWAP |11> = |11>```
///
/// Matrix form for [`SWAP`](swap()) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th></tr>
/// </table>
#[inline(always)]
pub fn swap(ab_mask: N,) -> MultiOp {
    swap::swap(ab_mask,)
        .expect("Mask should contain 2 bit!",)
        .into()
}

/// Square root of *SWAP* gate.
///
/// Performs a *half* SWAP of 2 qubits' state.
/// This gate is a natural way to couple qubits in some kinds of quantum systems.
///
/// ```sqrt(SWAP) * sqrt(SWAP) |q> = SWAP |q>```
/// 
/// Matrix form for [`sqrt(SWAP)`](sqrt_swap) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>(1+<i>i</i>)/2</th><th>(1-<i>i</i>)/2</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>(1-<i>i</i>)/2</th><th>(1+<i>i</i>)/2</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th></tr>
/// </table>
#[inline(always)]
pub fn sqrt_swap(ab_mask: N,) -> MultiOp {
    swap::sqrt_swap(ab_mask,)
        .expect("Mask should contain 2 bit!",)
        .into()
}

/// [`iSWAP`](i_swap) gate.
///
/// Perform SWAP of 2 qubits' state, multiplying bu *i* if qubits are not equals.
///
/// ```iSWAP |00> = |00>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```iSWAP |01> = i |01>```
///
/// ```iSWAP |10> = i |10>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```iSWAP |11> = |11>```
///
/// Matrix form for [`iSWAP`](i_swap) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;<i>i</i>&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;<i>i</i>&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th></tr>
/// </table>
#[inline(always)]
pub fn i_swap(ab_mask: N,) -> MultiOp {
    swap::i_swap(ab_mask,)
        .expect("Mask should contain 2 bit!",)
        .into()
}

/// Square root of *iSWAP* gate.
///
/// Performs a *half* iSWAP of 2 qubits' state.
/// This gate is a natural way to couple qubits in some kinds of quantum systems.
///
/// ```sqrt(iSWAP) * sqrt(iSWAP) |q> = iSWAP |q>```
/// 
/// Matrix form for [`sqrt(iSWAP)`](sqrt_i_swap) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th><th>&nbsp;&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>1/&radic;2</th><th><i>i</i>/&radic;2</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th><i>i</i>/&radic;2</th><th>1/&radic;2</th><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th></tr>
///     <tr><th>&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;&nbsp;0&nbsp;&nbsp;</th><th>&nbsp;&nbsp;1&nbsp;&nbsp;</th></tr>
/// </table>
#[inline(always)]
pub fn sqrt_i_swap(ab_mask: N,) -> MultiOp {
    swap::sqrt_i_swap(ab_mask,)
        .expect("Mask should contain 2 bit!",)
        .into()
}

/// Hadamard gate.
///
/// Performs Hadamard transform on a given qubits.
/// This is the simplest operation that create a superposition from a classical state.
/// It is similar to *flipping a coin*, which could fall on any side with an equal probability.
///
/// ```H |0> = |+> = ( |0> + |1> ) / sqrt(2)```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```H |1> = |-> = ( |0> - |1> ) / sqrt(2)```
///
/// States |+> and |-> are represent a *basis* on Bloch sphere, similar to |0> and |1> states.
///
/// Matrix form for [`H`](h()) gate:
///
/// <table cellpadding="10pt">
///     <tr><th>1/&radic;2</th><th>1/&radic;2</th></tr>
///     <tr><th>1/&radic;2</th><th>-1/&radic;2</th></tr>
/// </table>
///
/// [`H`](h()) gate acts on Bloch sphere:
///
/// | arbitrary state | state 0 | state 1 |
/// | :---: | :---: | :---: |
/// | ![H](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/H.gif) | ![H0](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/H0.gif) |  ![H1](https://raw.githubusercontent.com/MucTepDayH16/qvnt/master/animated/output/H1.gif) |
#[inline(always)]
pub fn h(a_mask: N,) -> MultiOp {
    h::h(a_mask,)
}

/// [`U1(λ)`](u1) gate.
///
/// First universal gate. Equivalent to [`RZ(λ)`](rz) and [`U3(0,0,λ)`](u3).
#[inline(always)]
pub fn u1(lam: R, a_mask: N,) -> MultiOp {
    rz(lam, a_mask,)
}

/// [`U2(φ,λ)`](u2) gate.
///
/// Second universal gate. Equivalent to [`U3(π/2,φ,λ)`](u3)
#[inline(always)]
pub fn u2(phi: R, lam: R, a_mask: N,) -> MultiOp {
    rz(lam, a_mask,) * ry(FRAC_PI_2, a_mask,) * rz(phi, a_mask,)
}

/// [`U3(θ,φ,λ)`](u3) gate.
///
/// Third universal gate.
///
/// Since unitary operators for a single qubit form the [`SU(2)`](https://en.wikipedia.org/wiki/Representation_theory_of_SU(2)) group,
/// they are described by only 3 free parameters.
/// This means that every single qubit gate can be represented in form of [`U3(θ,φ,λ)`](u3) with some global phase,
/// which does not affect qubit's behaviour.
///
/// ```X |q> = i U3(π,π,0) |q>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```Y |q> = i U3(π,0,0) |q>```
///
/// ```Z |q> = i U3(0,0,π) |q>```&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
/// ```H |q> = i U3(0,π/2,π) |q>```
#[inline(always)]
pub fn u3(the: R, phi: R, lam: R, a_mask: N,) -> MultiOp {
    rz(lam, a_mask,) * ry(the, a_mask,) * rz(phi, a_mask,)
}

/// Discrete Fourier transform ([`QFT`](qft())) for the quantum state's amplitudes.
///
/// Fourier transform with factor 1/&radic;N.
/// This factor is chosen so that [`QFT`](qft()) could be the unitary operator.
/// Quantum Fourier Transform consists of only O(n<sup>2</sup>) gates, where *n* is the number of qubits.
/// This can be compared with the classical Discrete Fourier Transform, which takes O(n2<sup>n</sup> gates,
/// which is exponentially more than [`QFT`](qft()).
#[inline(always)]
pub fn qft(a_mask: N,) -> MultiOp {
    qft::qft(a_mask,)
}

/// Discrete Fourier transform with qubits' swap
///
/// [`QFT`](qft()) is differ from real DFT by a bit order of amplitudes indices.
/// [`swapped QFT`](qft_swapped) is the more natural version of DFT.
#[inline(always)]
pub fn qft_swapped(a_mask: N,) -> MultiOp {
    qft::qft_swapped(a_mask,)
}

#[cfg(test)]
pub(crate) fn bench_circuit() -> MultiOp {
    MultiOp::default()
        * h(0b111,)
        * h(0b100,).c(0b001,).unwrap()
        * x(0b001,).c(0b110,).unwrap()
        * rx(1.2, 0b100,)
        * rz(1.0, 0b010,).c(0b001,).unwrap()
        * h(0b001,).c(0b100,).unwrap()
        * z(0b010,)
        * rxx(crate::math::FRAC_PI_6, 0b101,)
}
