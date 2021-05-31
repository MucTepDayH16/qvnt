use {
    std::{
        collections::{ BTreeMap, VecDeque, },
        boxed::Box,
        fmt,
        ops::{ Mul, MulAssign, },
        string::String,
        sync::{ Arc, RwLock }
    },

    crate::math::*,
};

pub(crate) struct Operator {
    pub(crate) name: String,
    pub(crate) control: Arc<usize>,
    pub(crate) func: Box<dyn Fn(&Vec<C>, N) -> C + Send + Sync>,
}

impl Operator {
    fn change_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c_mask = *self.control;
        write!(f, "{}",
               if c_mask != 0 { format!("C{}_", c_mask) } else { String::new() }
                   + &self.name)
    }
}

impl Into<Op> for Operator {
    fn into(self) -> Op {
        Op(VecDeque::from(vec![self]))
    }
}

macro_rules! simple_operator_definition {
    ($name:literal, $mask:expr, $operation:expr) => {{
        if $mask == 0 {
            Op::id()
        } else {
            Operator {
                name: format!("{}{}", $name, $mask),
                control: Arc::new(0),
                func: Box::new(move |psi, idx| $operation(psi, idx, $mask))
            }.into()
        }
    }};
}
macro_rules! rotate_operator_definition {
    ($name:literal, $dim:expr, $phase:expr, $mask:expr, $operation:expr) => {{
        let ang = phase_from_rad($phase * 0.5);
        let mut ops = Op::id();

        if ang != ANGLE_TABLE[0] {
            let mut real_mask = 0;
            for mask in crate::bits_iter::BitsIter::from($mask) {
                real_mask |= mask;
                if count_bits(real_mask) == $dim {
                    ops *= Operator {
                        name: format!("{}{}({})", $name, real_mask, $phase),
                        control: Arc::new(0),
                        func: Box::new(move |psi, idx| $operation(psi, idx, real_mask, ang))
                    };
                    real_mask = 0;
                }
            }
        }

        ops
    }};
}

#[derive(Debug)]
pub struct Op(pub(crate) VecDeque<Operator>);

impl Op {
    pub fn len(&self) -> N {
        self.0.len()
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Matrix form of a given operator.
    ///
    /// Return transponed matrix representation for the operator.
    ///
    /// ```rust
    /// use qvnt::prelude::*;
    /// use consts::{_0, _1, _i};
    /// assert_eq!(
    ///     Op::y(0b10).matrix_t::<4>(),
    ///     [   [_0,  _0, _i, _0],
    ///         [_0,  _0, _0, _i],
    ///         [-_i, _0, _0, _0],
    ///         [_0, -_i, _0, _0]]
    /// );
    /// ```
    #[cfg(test)]
    pub fn matrix_t<const Q_SIZE: usize>(&self) -> [[C; Q_SIZE]; Q_SIZE] {
        assert_eq!(Q_SIZE.count_ones(), 1);
        assert_ne!(Q_SIZE & 0b11111, 0);
        let mut matrix = [[C::zero(); Q_SIZE]; Q_SIZE];
        for b in 0..Q_SIZE {
            let mut reg = crate::register::QReg::new(5).init_state(b);
            reg.apply(self);
            unsafe { matrix[b].clone_from_slice(&reg.psi[0..Q_SIZE]) };
        }
        matrix
    }

    /// Identity operator.
    ///
    /// For any quantum state |q>, identity operator does not change state.
    ///
    /// I |q> = |q>
    #[inline(always)]
    pub fn id() -> Self {
        Self(VecDeque::new())
    }

    /// Controlled version of operator.
    ///
    /// Change the operator to one controlled by given qubits.
    ///
    /// ```rust
    /// use qvnt::prelude::*;
    /// // Get *X* Pauli operator, aka *NOT* gate, acting on first qubit.
    /// let usual_op = Op::x(0b001);
    /// // Get this operator, controlled by second and third qubit.
    /// // This operator is the Toffoli gate, *aka* CCNot gate.
    /// // Previous *usual_op* operator is consumed.
    /// let contr_op = usual_op.c(0b110);
    /// ```
    pub fn c(mut self, c_mask: N) -> Self {
        self.0.iter_mut().for_each(move |op|
            op.control = Arc::new(*op.control | c_mask)
        );
        self
    }

    /// Pauli *X* operator, *aka* NOT gate.
    ///
    /// Performs negation for given qubits.
    ///
    /// X |0> = |1>
    ///
    /// X |1> = |0>
    pub fn x(a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N) -> C {
            psi[idx ^ a_mask]
        }
        simple_operator_definition!("X", a_mask, _op)
    }
    /// *X* rotation operator.
    ///
    /// Performs *phase* radians rotation around X axis on a Bloch sphere.
    pub fn rx(phase: R, a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ a_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            psi.0.scale(ang.re) + psi.1.scale(ang.im)
        }
        rotate_operator_definition!("RX", 1, phase, a_mask, _op)
    }
    /// *Ising XX* coupling gate.
    ///
    /// Performs *phase* radians rotation around XX axis on 2-qubit Bloch spheres.
    pub fn rxx(phase: R, ab_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ ab_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RXX", 2, phase, ab_mask, _op)
    }

    pub fn y(a_mask: N) -> Self {
        let i = I_POW_TABLE[ (!count_bits(a_mask)).wrapping_add(1) & 0x3 ];
        if a_mask == 0 {
            Self::id()
        } else {
            Operator {
                name: format!("{}{}", "Y", a_mask),
                control: Arc::new(0),
                func: Box::new(move |psi, idx| -> C {
                    i * if count_bits(idx & a_mask).is_odd() {
                        -psi[idx ^ a_mask]
                    } else {
                        psi[idx ^ a_mask]
                    }
                })
            }.into()
        }
    }
    /// *Y* rotation operator.
    ///
    /// Performs *phase* radians rotation around Y axis on a Bloch sphere.
    pub fn ry(phase: R, a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, mut ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ a_mask]);
            if idx & a_mask == 0 { ang.im = -ang.im; }
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RY", 1, phase, a_mask, _op)
    }
    /// *Ising YY* coupling gate.
    ///
    /// Performs *phase* radians rotation around YY axis on 2-qubit Bloch spheres.
    pub fn ryy(phase: R, ab_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, mut ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ ab_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            if (idx & ab_mask).count_ones().is_even() { ang.im = -ang.im; }
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RYY", 2, phase, ab_mask, _op)
    }

    /// Pauli *Z* operator.
    ///
    /// Negate an amplitude of 1-state.
    ///
    /// Z |0> = |0>
    ///
    /// Z |1> = -|1>
    pub fn z(a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N) -> C {
            if count_bits(idx & a_mask).is_odd() {
                -psi[idx]
            } else {
                psi[idx]
            }
        }
        simple_operator_definition!("Z", a_mask, _op)
    }
    /// *S* operator.
    ///
    /// Square root of *Z* operator.
    ///
    /// S |0> = |0>
    ///
    /// S |1> = i|1>
    pub fn s(a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N) -> C {
            I_POW_TABLE[count_bits(idx & a_mask) & 3] * psi[idx]
        }
        simple_operator_definition!("S", a_mask, _op)
    }
    /// *S* operator.
    ///
    /// Fourth root of *Z* operator.
    ///
    /// T |0> = |0>
    ///
    /// T |1> = (1+i)/sqrt(2) |1>
    pub fn t(a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N) -> C {
            let count = count_bits(idx & a_mask);
            (if count & 1 != 0 { ANGLE_TABLE[45] } else { C::one() })
                * I_POW_TABLE[(count >> 1) & 3] * psi[idx]
        }
        simple_operator_definition!("T", a_mask, _op)
    }
    /// *Z* rotation operator.
    ///
    /// Performs *phase* radians rotation around Z axis on a Bloch sphere.
    pub fn rz(phase: R, a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, mut ang: C) -> C {
            let mut psi = psi[idx];
            if idx & a_mask == 0 { ang.im = -ang.im; }
            ang * psi
        }
        rotate_operator_definition!("RZ", 1, phase, a_mask, _op)
    }
    /// *Ising ZZ* coupling gate.
    ///
    /// Performs *phase* radians rotation around ZZ axis on 2-qubit Bloch spheres.
    pub fn rzz(phase: R, ab_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, mut ang: C) -> C {
            let mut psi = psi[idx];
            if (idx & ab_mask).count_ones().is_even() { ang.im = -ang.im; }
            ang * psi
        }
        rotate_operator_definition!("RZZ", 2, phase, ab_mask, _op)
    }

    /// Phase shift operator.
    ///
    /// Performs phase shift for a range of given qubits by corresponding phase.
    ///
    /// ```rust
    /// use qvnt::prelude::*;
    /// use std::f64::consts::PI;
    /// //  Take a third root of *Z* gate.
    /// let z_pow_a = Op::phi(vec![(PI / 3., 0b1)]);
    /// //  Equivalent to Op::z(0b1).
    /// let z = Op::phi(vec![(PI, 0b1)]);
    /// ```
    pub fn phi(angles_vec: Vec<(R, N)>) -> Self {
        let mut angles = BTreeMap::new();
        for (val, idx) in angles_vec.iter() {
            let mut jdx = 1;
            while jdx <= *idx {
                if jdx & *idx != 0 {
                    angles.entry(jdx).or_insert(C::new(0.0, 0.0)).im += *val;
                }
                jdx <<= 1;
            }
        }
        angles.iter_mut().for_each(|(idx, val)| *val = C::from_polar(1.0, val.im));

        if angles.is_empty() {
            Self::id()
        } else {
            Operator {
                name: format!("Phase{:?}", angles_vec),
                control: Arc::new(0),
                func: Box::new(
                    move |psi, idx| {
                        let mut val = psi[idx];
                        for (jdx, ang) in &angles {
                            if idx & jdx != 0 {
                                val *= ang;
                            }
                        }
                        val
                    }
                )
            }.into()
        }
    }

    /// *SWAP* gate.
    ///
    /// Performs SWAP of 2 qubits' value.
    ///
    /// SWAP |ab> = |ba>
    pub fn swap(ab_mask: N) -> Self {
        assert_eq!(ab_mask.count_ones(), 2);
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N) -> C {
            if (idx & ab_mask).count_ones().is_odd() {
                psi[idx ^ ab_mask]
            } else {
                psi[idx]
            }
        }
        simple_operator_definition!("SWAP", ab_mask, _op)
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
    /// use consts::*;
    ///
    /// //  sqrt(SWAP) gate's matrix representation:
    /// assert_eq!(
    ///     Op::sqrt_swap(0b11).matrix_t::<4>(),
    ///     [   [_1, _0,              _0,              _0],
    ///         [_0, 0.5 * (_1 + _i), 0.5 * (_1 - _i), _0],
    ///         [_0, 0.5 * (_1 - _i), 0.5 * (_1 + _i), _0],
    ///         [_0, _0,              _0,              _1]]
    /// );
    /// ```
    pub fn sqrt_swap(ab_mask: N) -> Self {
        assert_eq!(ab_mask.count_ones(), 2);
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N) -> C {
            if (idx & ab_mask).count_ones().is_odd() {
                let psi = (psi[idx], psi[idx ^ ab_mask]);
                0.5 * C {
                    re: (psi.0.re - psi.0.im) + (psi.1.re + psi.1.im),
                    im: (psi.0.im + psi.0.re) + (psi.1.im - psi.1.re)
                }
            } else {
                psi[idx]
            }
        }
        simple_operator_definition!("sqrt_SWAP", ab_mask, _op)
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
    pub fn i_swap(ab_mask: N) -> Self {
        assert_eq!(ab_mask.count_ones(), 2);
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N) -> C {
            if (idx & ab_mask).count_ones().is_odd() {
                let psi = psi[idx ^ ab_mask];
                C { re: -psi.im, im: psi.re }
            } else {
                psi[idx]
            }
        }
        simple_operator_definition!("iSWAP", ab_mask, _op)
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
    /// use consts::*;
    ///
    /// //  sqrt(iSWAP) gate's matrix representation:
    /// assert_eq!(
    ///     Op::sqrt_swap(0b11).matrix_t::<4>(),
    ///     [   [_1, _0,            _0,            _0],
    ///         [_0, SQRT_1_2 * _1, SQRT_1_2 * _i, _0],
    ///         [_0, SQRT_1_2 * _i, SQRT_1_2 * _1, _0],
    ///         [_0, _0,            _0,            _1]]
    /// );
    /// ```
    pub fn sqrt_i_swap(ab_mask: N) -> Self {
        assert_eq!(ab_mask.count_ones(), 2);
        const SQRT_1_2: R = SQRT_2 * 0.5;

        Operator {
            name: format!("sqrt_iSWAP{}", ab_mask),
            control: Arc::new(0),
            func: Box::new(
                move |psi, mut idx| {
                    if (idx & ab_mask).count_ones().is_odd() {
                        let psi = (psi[idx], psi[idx ^ ab_mask]);
                        SQRT_1_2 * C { re: psi.0.re - psi.1.im, im: psi.0.im + psi.1.re }
                    } else {
                        psi[idx]
                    }
                }
            )
        }.into()
    }

    /// Hadamard gate.
    ///
    /// Performs Hadamard transform on a given qubits.
    /// This is the simplest operation that create a superposition from a pure state |i>
    ///
    /// H |0> = |+> = ( |0> + |1> ) / sqrt(2)
    ///
    /// H |1> = |-> = ( |0> - |1> ) / sqrt(2)
    pub fn h(a_mask: N) -> Self {
        fn _op_1x1(a_mask: N) -> Operator {
            {
                assert_eq!(a_mask.count_ones(), 1);
            }

            const SQRT_1_2: R = SQRT_2 * 0.5;

            Operator {
                name: format!("H{}", a_mask),
                control: Arc::new(0),
                func: Box::new(
                    move |psi, idx| {
                        let a = (idx & a_mask) != 0;
                        (if a { -psi[idx] } else { psi[idx] } + psi[idx ^ a_mask]) * SQRT_1_2
                    }
                )
            }
        }
        fn _op_2x2(a_mask: N, b_mask: N) -> Operator {
            {
                assert_eq!(a_mask.count_ones(), 1);
                assert_eq!(b_mask.count_ones(), 1);
                assert_eq!(a_mask & b_mask, 0);
            }

            let ab_mask = a_mask | b_mask;

            Operator {
                name: format!("H{}", ab_mask),
                control: Arc::new(0),
                func: Box::new(
                    move |psi, idx| {
                        let a = (idx & a_mask) != 0;
                        let b = (idx & b_mask) != 0;
                        (   if a ^ b { -psi[idx] } else { psi[idx] }                +
                            if b { -psi[idx ^ a_mask] } else { psi[idx ^ a_mask] }  +
                            if a { -psi[idx ^ b_mask] } else { psi[idx ^ b_mask] }  +
                            psi[idx ^ ab_mask]                                      ) * 0.5
                    }
                )
            }
        }

        let count = count_bits(a_mask);

        match count {
            0 => Self::id(),
            1 => _op_1x1(a_mask).into(),
            _ => {
                let mut res = Op(VecDeque::with_capacity((count + 1) >> 1));
                let mut idx = (1, 0);
                let mut is_first = true;

                while idx.0 <= a_mask {
                    if idx.0 & a_mask != 0 {
                        if is_first {
                            idx.1 = idx.0;
                            is_first = false;
                        } else {
                            res.0.push_back(_op_2x2(idx.0, idx.1));
                            is_first = true;
                        }
                    }
                    idx.0 <<= 1;
                }

                if !is_first {
                    res.0.push_back(_op_1x1(idx.1));
                }

                res
            }
        }
    }

    /// *U1(lam)* gate.
    ///
    /// First universal operator. Equivalent to *RZ* and U3(0,0,lam).
    pub fn u1(lam: R, a_mask: N) -> Self {
        Self::rz(lam, a_mask)
    }
    /// *U2(phi,lam)* gate.
    ///
    /// Second universal operator. Equivalent to U3(PI/2, phi, lam)
    pub fn u2(phi: R, lam: R, a_mask: N) -> Self {
        Self::rz(lam + PI, a_mask)
            * Self::h(a_mask)
            * Self::rz(phi, a_mask)
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
    pub fn u3(the: R, phi: R, lam: R, a_mask: N) -> Self {
        Self::rz(lam, a_mask)
            * Self::ry(the, a_mask)
            * Self::rz(phi, a_mask)
    }
    pub(crate) fn uni_1x1(u: M1, a_mask: N) -> Self {
        assert_eq!(a_mask.count_ones(), 1);
        assert!(is_unitary_m1(&u));

        if is_diagonal_m1(&u) {
            Operator {
                name: format!("Diag[{:?}, {:?}]", u[0], u[3]),
                control: Arc::new(0),
                func: Box::new(
                    move |psi, idx|
                        u[if (idx & a_mask) != 0 { 3 } else { 0 }] * psi[idx]
                )
            }
        } else {
            Operator {
                name: format!("Unit{:?}", u),
                control: Arc::new(0),
                func: Box::new(
                    move |psi, idx| {
                        let udx = if (idx & a_mask) != 0 { (3, 2) } else { (0, 1) };
                        u[udx.0] * psi[idx] + u[udx.1] * psi[idx ^ a_mask]
                    }
                )
            }
        }.into()
    }
    pub(crate) fn uni_2x2(u: M2, a_mask: N, b_mask: N) -> Self {
        assert_eq!(a_mask.count_ones(), 1);
        assert_eq!(b_mask.count_ones(), 1);
        assert_eq!(a_mask & b_mask, 0);
        assert!(is_unitary_m2(&u));

        let ab_mask = a_mask | b_mask;

        if is_diagonal_m2(&u) {
            Operator {
                name: format!("Diag[{:?}, {:?}, {:?}, {:?}]", u[0b0000], u[0b0101], u[0b1010], u[0b1111]),
                control: Arc::new(0),
                func: Box::new(
                    move |psi, idx| {
                        let udx =
                            if (idx & a_mask) != 0 { 0b0101 } else { 0b0000 }
                            | if (idx & b_mask) != 0 { 0b1010 } else { 0b0000 };
                        u[udx] * psi[idx]
                    }
                )
            }
        } else {
            Operator {
                name: format!("Unit{:?}", u),
                control: Arc::new(0),
                func: Box::new(
                    move |psi, idx| {
                        let udx =
                            if (idx & a_mask) != 0 { 0b0101 } else { 0b0000 } |
                            if (idx & b_mask) != 0 { 0b1010 } else { 0b0000 };
                        (u[udx ^ 0] * psi[idx] + u[udx ^ 1] * psi[idx ^ a_mask]) +
                        (u[udx ^ 2] * psi[idx ^ b_mask] + u[udx ^ 3] * psi[idx ^ ab_mask])
                    }
                )
            }
        }.into()
    }
    pub(crate) fn if_b_then_u1_else_u0(u0: M1, u1: M1, a_mask: N, b_mask: N) -> Self {
        let mut u = [C::zero(); 16];
        u[0b0000] = u0[0b00];
        u[0b0001] = u0[0b01];
        u[0b0100] = u0[0b10];
        u[0b0101] = u0[0b11];
        u[0b1010] = u1[0b00];
        u[0b1011] = u1[0b01];
        u[0b1110] = u1[0b10];
        u[0b1111] = u1[0b11];
        Self::uni_2x2(u, a_mask, b_mask)
    }

    /// Discrete Fourier transform for the quantum state's amplitudes.
    ///
    /// Fourier transform with factor 1/sqrt(N).
    /// This transform keeps the norm of vector, so it could be applied as unitary operator.
    /// It use the technique of fast fourier transform and have O(n*log(n)) time complexity.
    ///
    /// Fourier transform on a single qubit is just a Hadamard gate.
    pub fn qft(a_mask: N) -> Self {
        let count = a_mask.count_ones() as usize;
        match count {
            0 => Self::id(),
            1 => Self::h(a_mask),
            _ => {
                let mut res = VecDeque::new();
                let mut vec = Vec::<usize>::with_capacity(count);

                for idx in 0..64 {
                    let jdx = 1 << idx;
                    if jdx & a_mask != 0 {
                        vec.push(jdx);
                    }
                }

                for i in 0..(count-1) {
                    res.append(&mut Op::h(vec[i]).0);
                    res.append(
                        &mut Op::phi(
                            ((i+1)..count).map(|j| (PI * (0.5 as R).pow((j-i) as u8), vec[j]) ).collect())
                            .c(vec[i]).0
                    );
                }

                res.append(&mut Op::h(vec[count-1]).0);
                Op(res)
            }
        }
    }
    /// Discrete Fourier transform with qubits' swap
    ///
    /// QFT is differ from real DFT by a bit order of amplitudes indices.
    /// *qft_swapped* is a natural version of DFT.
    pub fn qft_swapped(a_mask: N) -> Self {
        let mut vec_mask = Vec::with_capacity(count_bits(a_mask));
        let mut idx = 1;
        while idx <= a_mask {
            if idx & a_mask != 0 {
                vec_mask.push(idx);
            }
            idx <<= 1;
        }

        let mut swaps = Op::id();
        let len = vec_mask.len();
        for i in 0..(len >> 1) {
            swaps *= Op::swap(vec_mask[i] | vec_mask[len - i - 1]);
        }

        Self::qft(a_mask) * swaps
    }

    #[cfg(test)]
    pub(crate) fn bench_circuit() -> Self {
        Op::id()
            * Op::h(0b111)
            * Op::h(0b100).c(0b001)
            * Op::x(0b001).c(0b110)
            * Op::rx(1.2, 0b100)
            * Op::phi(vec![ (1.0, 0b010) ]).c(0b001)
            * Op::h(0b001).c(0b100)
            * Op::z(0b010)
            * Op::rxx(FRAC_PI_6, 0b101)
    }
}

impl Default for Op {
    fn default() -> Self {
        Self::id()
    }
}

impl Mul for Op {
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self {
        self.mul_assign(rhs);
        self
    }
}

impl Mul<Operator> for Op {
    type Output = Self;

    fn mul(mut self, mut rhs: Operator) -> Self {
        self.mul_assign(rhs);
        self
    }
}

impl<'a> Mul<Op> for &'a mut Op {
    type Output = Self;

    fn mul(self, mut rhs: Op) -> Self::Output {
        self.mul_assign(rhs);
        self
    }
}

impl MulAssign for Op {
    fn mul_assign(&mut self, mut rhs: Self) {
        self.0.append(&mut rhs.0);
    }
}

impl MulAssign<Operator> for Op {
    fn mul_assign(&mut self, mut rhs: Operator) {
        self.0.push_back(rhs);
    }
}

impl<'a> MulAssign<Op> for &'a mut Op {
    fn mul_assign(&mut self, mut rhs: Op) {
        self.0.append(&mut rhs.0);
    }
}