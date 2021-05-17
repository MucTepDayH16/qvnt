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
        assert_eq!($mask.count_ones(), $dim);
        let ang = phase_from_rad($phase * 0.5);

        if ang == ANGLE_TABLE[0] {
            Op::id()
        } else {
            Operator {
                name: format!("{}{}({})", $name, $mask, $phase),
                control: Arc::new(0),
                func: Box::new(move |psi, idx| $operation(psi, idx, $mask, ang))
            }.into()
        }
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

    pub fn id() -> Self {
        Self(VecDeque::new())
    }

    pub fn c(mut self, c_mask: N) -> Self {
        self.0.iter_mut().for_each(move |op|
            op.control = Arc::new(*op.control | c_mask)
        );
        self
    }

    pub fn x(a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N) -> C {
            psi[idx ^ a_mask]
        }
        simple_operator_definition!("X", a_mask, _op)
    }
    pub fn rx(phase: R, a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ a_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RX", 1, phase, a_mask, _op)
    }
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
    pub fn ry(phase: R, a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, mut ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ a_mask]);
            if idx & a_mask == 0 { ang.im = -ang.im; }
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RY", 1, phase, a_mask, _op)
    }
    pub fn ryy(phase: R, ab_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, mut ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ ab_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            if (idx & ab_mask).count_ones().is_even() { ang.im = -ang.im; }
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RYY", 2, phase, ab_mask, _op)
    }

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
    pub fn s(a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N) -> C {
            I_POW_TABLE[count_bits(idx & a_mask) & 3] * psi[idx]
        }
        simple_operator_definition!("S", a_mask, _op)
    }
    pub fn t(a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N) -> C {
            let count = count_bits(idx & a_mask);
            (if count & 1 != 0 { ANGLE_TABLE[45] } else { C::one() })
                * I_POW_TABLE[(count >> 1) & 3] * psi[idx]
        }
        simple_operator_definition!("T", a_mask, _op)
    }
    pub fn rz(phase: R, a_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, mut ang: C) -> C {
            let mut psi = psi[idx];
            if idx & a_mask == 0 { ang.im = -ang.im; }
            ang * psi
        }
        rotate_operator_definition!("RZ", 1, phase, a_mask, _op)
    }
    pub fn rzz(phase: R, ab_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, mut ang: C) -> C {
            let mut psi = psi[idx];
            if (idx & ab_mask).count_ones().is_even() { ang.im = -ang.im; }
            ang * psi
        }
        rotate_operator_definition!("RZZ", 2, phase, ab_mask, _op)
    }

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

    pub fn uni_1x1(u: M1, a_mask: N) -> Self {
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
    pub fn uni_2x2(u: M2, a_mask: N, b_mask: N) -> Self {
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
    pub fn if_b_then_u1_else_u0(u0: M1, u1: M1, a_mask: N, b_mask: N) -> Self {
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

    pub fn qft(a_mask: N) -> Self {
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

        Self::qft_no_swap(a_mask) * swaps
    }
    pub fn qft_no_swap(a_mask: N) -> Self {
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

    pub fn bench_circuit() -> Self {
        Op::id()
            * Op::h(0b111)
            * Op::h(0b100).c(0b001)
            * Op::x(0b001).c(0b110)
            * Op::ry(1.2, 0b100)
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

    fn mul(mut self, mut rhs: Self) -> Self::Output {
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

impl<'a> MulAssign<Op> for &'a mut Op {
    fn mul_assign(&mut self, mut rhs: Op) {
        self.0.append(&mut rhs.0);
    }
}