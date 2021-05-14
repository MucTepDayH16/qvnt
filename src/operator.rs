use {
    std::{
        collections::{
            BTreeMap,
            VecDeque,
        },
        boxed::Box,
        fmt,
        ops::{
            Mul,
            MulAssign,
        },
        string::String
    },

    crate::math::*,
};

pub(crate) struct Operator {
    pub(crate) name: String,
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
        write!(f, "{}", self.name)
    }
}

impl Into<Op> for Operator {
    fn into(self) -> Op {
        Op(VecDeque::from(vec![self]))
    }
}

macro_rules! simple_operator_definition {
    ($name:literal, $mask:expr, $c_mask:expr, $operation:expr) => {{
        let mask = $mask & !$c_mask;
        let name = format!("{}{}", $name, mask);

        if mask == 0 {
            Op::id()
        } else if $c_mask == 0 {
            Operator {
                name, func: Box::new(
                    move |psi, idx|
                        $operation(psi, idx, mask)
                )
            }.into()
        } else {
            let name = format!("C{}_", $c_mask) + &name;
            Operator {
                name, func: Box::new(
                    move |psi, idx|
                        if !idx & $c_mask == 0 {
                            $operation(psi, idx, mask)
                        } else {
                            psi[idx]
                        }
                )
            }.into()
        }
    }};
}
macro_rules! rotate_operator_definition {
    ($name:literal, $dim:expr, $phase:expr, $mask:expr, $c_mask:expr, $operation:expr) => {{
        let mask = $mask & !$c_mask;
        assert_eq!(mask.count_ones(), $dim);
        let ang = phase_from_rad($phase * 0.5);
        let name = format!("{}{}[{}]", $name, mask, $phase);

        if ang == ANGLE_TABLE[0] {
            Op::id()
        } else if $c_mask == 0 {
            Operator {
                name, func: Box::new(
                    move |psi, idx|
                        $operation(psi, idx, mask, ang)
                )
            }.into()
        } else {
            let name = format!("C{}_", $c_mask) + &name;
            Operator {
                name, func: Box::new(
                    move |psi, idx|
                        if !idx & $c_mask == 0 {
                            $operation(psi, idx, mask, ang)
                        } else {
                            psi[idx]
                        }
                )
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

    pub fn x(a_mask: N, c_mask: N) -> Self {
        let a_mask = a_mask & !c_mask;

        if a_mask == 0 {
            Self::id()
        } else if c_mask == 0 {
            Operator {
                name: format!("X{}", a_mask),
                func: Box::new(
                    move |psi, idx|
                        psi[idx ^ a_mask]
                )
            }.into()
        } else {
            Operator {
                name: format!("C{}_X{}", c_mask, a_mask),
                func: Box::new(
                    move |psi, idx|
                        psi[if !idx & c_mask == 0 { idx ^ a_mask } else { idx }]
                )
            }.into()
        }
    }
    pub fn rx(phase: R, a_mask: N, c_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ a_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RX", 1, phase, a_mask, c_mask, _op)
    }
    pub fn rxx(phase: R, ab_mask: N, c_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ ab_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RXX", 2, phase, ab_mask, c_mask, _op)
    }

    pub fn y(a_mask: N, c_mask: N) -> Self {
        let a_mask = a_mask & !c_mask;
        let i = I_POW_TABLE[ (!count_bits(a_mask)).wrapping_add(1) & 0x3 ];

        if a_mask == 0 {
            Self::id()
        } else if c_mask == 0 {
            Operator {
                name: format!("Y{}", a_mask),
                func: Box::new(
                    move |psi, idx|
                        i * if count_bits(idx & a_mask).is_odd() {
                            -psi[idx ^ a_mask]
                        } else {
                            psi[idx ^ a_mask]
                        }
                )
            }.into()
        } else {
            Operator {
                name: format!("X_c{}_a{}", c_mask, a_mask),
                func: Box::new(
                    move |psi, idx|
                        if !idx & c_mask == 0 {
                            i * if count_bits(idx & a_mask).is_odd() {
                                -psi[idx ^ a_mask]
                            } else {
                                psi[idx ^ a_mask]
                            }
                        } else {
                            psi[idx]
                        }
                )
            }.into()
        }
    }
    pub fn ry(phase: R, a_mask: N, c_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, mut ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ a_mask]);
            if idx & a_mask != 0 { ang.im = -ang.im; }
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RY", 1, phase, a_mask, c_mask, _op)
    }
    pub fn ryy(phase: R, ab_mask: N, c_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, mut ang: C) -> C {
            let mut psi = (psi[idx], psi[idx ^ ab_mask]);
            psi.1 = C::new(psi.1.im, -psi.1.re);
            if (idx & ab_mask).count_ones().is_even() { ang.im = -ang.im; }
            ang.re * psi.0 + ang.im * psi.1
        }
        rotate_operator_definition!("RYY", 2, phase, ab_mask, c_mask, _op)
    }

    pub fn z(a_mask: N, c_mask: N) -> Self {
        let a_mask = a_mask & !c_mask;

        if a_mask == 0 {
            Self::id()
        } else if c_mask == 0 {
            Operator {
                name: format!("Z_a{}", a_mask),
                func: Box::new(
                    move |psi, idx|
                        if count_bits(idx & a_mask).is_odd() {
                            -psi[idx]
                        } else {
                            psi[idx]
                        }
                )
            }.into()
        } else {
            Operator {
                name: format!("Z_c{}_a{}", c_mask, a_mask),
                func: Box::new(
                    move |psi, idx|
                        if !idx & c_mask == 0 && count_bits(idx & a_mask).is_odd() {
                            -psi[idx]
                        } else {
                            psi[idx]
                        }
                )
            }.into()
        }
    }
    pub fn rz(phase: R, a_mask: N, c_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, a_mask: N, mut ang: C) -> C {
            let mut psi = psi[idx];
            if idx & a_mask == 0 { ang.im = -ang.im; }
            ang * psi
        }
        rotate_operator_definition!("RZ", 1, phase, a_mask, c_mask, _op)
    }
    pub fn rzz(phase: R, ab_mask: N, c_mask: N) -> Self {
        #[inline(always)] fn _op(psi: &[C], idx: N, ab_mask: N, mut ang: C) -> C {
            let mut psi = psi[idx];
            if (idx & ab_mask).count_ones().is_even() { ang.im = -ang.im; }
            ang * psi
        }
        rotate_operator_definition!("RZZ", 2, phase, ab_mask, c_mask, _op)
    }

    pub fn phi( angles_vec: Vec<(N, R)>, c_mask: N ) -> Self {
        let angles = angles_vec.clone().into_iter().filter_map(
            |(jdx, ang)|
                if c_mask & jdx != 0 {
                    None
                } else {
                    let phase = phase_from_rad(ang);
                    if phase == ANGLE_TABLE[0] {
                        None
                    } else {
                        Some((jdx, phase))
                    }
                }
        ).collect::<BTreeMap<N, C>>();

        if angles.is_empty() { return Self::id(); }

        Operator {
            name: if c_mask == 0 {
                format!("RZ_{:?}", angles_vec)
            } else {
                format!("RZ_c{}_{:?}", c_mask, angles_vec)
            },
            func: Box::new(
                move |psi, idx| {
                    let mut val = psi[idx];
                    if !idx & c_mask == 0 {
                        for (jdx, ang) in &angles {
                            let count = ((idx ^ c_mask) & jdx).count_ones();
                            if count > 0 {
                                val *= ang.pow(count);
                            }
                        }
                    }
                    val
                }
            )
        }.into()
    }

    pub fn swap(a_mask: N, b_mask: N, c_mask: N) -> Self {
        let a_mask = a_mask & !c_mask;
        let b_mask = b_mask & !c_mask;
        let ab_mask = a_mask | b_mask;

        assert_eq!(a_mask.count_ones(), 1);
        assert_eq!(b_mask.count_ones(), 1);
        assert_eq!(a_mask & b_mask, 0);

        if c_mask == 0 {
            Operator {
                name: format!("SWAP{}", ab_mask),
                func: Box::new(
                    move |psi, mut idx| {
                        if (idx & a_mask != 0) ^ (idx & b_mask != 0) {
                            psi[idx ^ ab_mask]
                        } else {
                            psi[idx]
                        }
                    }
                )
            }.into()
        } else {
            Operator {
                name: format!("C{}_SWAP{}", c_mask, ab_mask),
                func: Box::new(
                    move |psi, mut idx| {
                        if ((idx & a_mask != 0) ^ (idx & b_mask != 0)) && !idx & c_mask == 0 {
                            psi[idx ^ ab_mask]
                        } else {
                            psi[idx]
                        }
                    }
                )
            }.into()
        }
    }

    pub fn h(a_mask: N, c_mask: N) -> Self {
        fn _op_1x1(a_mask: N, c_mask: N) -> Operator {
            {
                assert_eq!(a_mask.count_ones(), 1);
                assert_eq!(a_mask & c_mask, 0);
            }

            const SQRT_1_2: R = SQRT_2 * 0.5;

            if c_mask == 0 {
                Operator {
                    name: format!("H_a{}", a_mask),
                    func: Box::new(
                        move |psi, idx| {
                            let a = (idx & a_mask) != 0;
                            (if a { -psi[idx] } else { psi[idx] } + psi[idx ^ a_mask]) * SQRT_1_2
                        }
                    )
                }
            } else {
                Operator {
                    name: format!("H_a{}", a_mask),
                    func: Box::new(
                        move |psi, idx|
                            if !idx & c_mask == 0 {
                                let a = (idx & a_mask) != 0;
                                (if a { -psi[idx] } else { psi[idx] } + psi[idx ^ a_mask]) * SQRT_1_2
                            } else {
                                psi[idx]
                            }
                    )
                }
            }
        }
        fn _op_2x2(a_mask: N, b_mask: N, c_mask: N) -> Operator {
            {
                assert_eq!(a_mask.count_ones(), 1);
                assert_eq!(b_mask.count_ones(), 1);
                assert_eq!(a_mask & b_mask, 0);
                assert_eq!(a_mask & c_mask, 0);
                assert_eq!(b_mask & c_mask, 0);
            }

            let ab_mask = a_mask | b_mask;

            if c_mask == 0 {
                Operator {
                    name: format!("H_a{}", ab_mask),
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
            } else {
                Operator {
                    name: format!("H_a{}", ab_mask),
                    func: Box::new(
                        move |psi, idx|
                            if !idx & c_mask == 0 {
                                let a = (idx & a_mask) != 0;
                                let b = (idx & b_mask) != 0;
                                (   if a ^ b { -psi[idx] } else { psi[idx] }                +
                                    if b { -psi[idx ^ a_mask] } else { psi[idx ^ a_mask] }  +
                                    if a { -psi[idx ^ b_mask] } else { psi[idx ^ b_mask] }  +
                                    psi[idx ^ ab_mask]                                      ) * 0.5
                            } else {
                                psi[idx]
                            }
                    )
                }
            }
        }

        let a_mask = a_mask & !c_mask;
        let count = count_bits(a_mask);

        match count {
            0 => Self::id(),
            1 => _op_1x1(a_mask, c_mask).into(),
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
                            res.0.push_back(_op_2x2(idx.0, idx.1, c_mask));
                            is_first = true;
                        }
                    }
                    idx.0 <<= 1;
                }

                if !is_first {
                    res.0.push_back(_op_1x1(idx.1, c_mask));
                }

                res
            }
        }
    }

    pub fn uni_1x1(u: M1, a_mask: N, c_mask: N) -> Self {
        let a_mask = a_mask & !c_mask;
        assert_eq!(a_mask.count_ones(), 1);
        assert!(is_unitary_m1(&u));

        if is_diagonal_m1(&u) {
            Operator {
                name: format!("Diag[{:?}, {:?}]", u[0], u[3]),
                func: Box::new(
                    move |psi, idx|
                        u[if (idx & a_mask) != 0 { 3 } else { 0 }] * psi[idx]
                )
            }
        } else {
            Operator {
                name: format!("Unitary{:?}", u),
                func: Box::new(
                    move |psi, idx| {
                        let udx = if (idx & a_mask) != 0 { (3, 2) } else { (0, 1) };
                        u[udx.0] * psi[idx] + u[udx.1] * psi[idx ^ a_mask]
                    }
                )
            }
        }.into()
    }
    pub fn uni_2x2(u: M2, a_mask: N, b_mask: N, c_mask: N) -> Self {
        let a_mask = a_mask & !c_mask;
        let b_mask = b_mask & !c_mask;
        let ab_mask = a_mask | b_mask;
        assert_eq!(a_mask.count_ones(), 1);
        assert_eq!(b_mask.count_ones(), 1);
        assert_eq!(a_mask & b_mask, 0);
        assert!(is_unitary_m2(&u));

        if is_diagonal_m2(&u) {
            Operator {
                name: format!("Diag[{:?}, {:?}, {:?}, {:?}]", u[0b0000], u[0b0101], u[0b1010], u[0b1111]),
                func: Box::new(
                    move |psi, idx| {
                        let udx =
                            if (idx & a_mask) != 0 { 0b0101 } else { 0b0000 } |
                            if (idx & b_mask) != 0 { 0b1010 } else { 0b0000 };
                        u[udx] * psi[idx]
                    }
                )
            }
        } else {
            Operator {
                name: format!("Unitary{:?}", u),
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
    pub fn if_b_then_u1_else_u0(u0: M1, u1: M1, a_mask: N, b_mask: N, c_mask: N) -> Self {
        let mut u = [C::zero(); 16];
        u[0b0000] = u0[0b00];
        u[0b0001] = u0[0b01];
        u[0b0100] = u0[0b10];
        u[0b0101] = u0[0b11];
        u[0b1010] = u1[0b00];
        u[0b1011] = u1[0b01];
        u[0b1110] = u1[0b10];
        u[0b1111] = u1[0b11];
        Self::uni_2x2(u, a_mask, b_mask, c_mask)
    }

    pub fn qft_no_swap(a_mask: N) -> Self {
        let count = a_mask.count_ones() as usize;
        match count {
            0 => Self::id(),
            1 => Self::h(a_mask, 0),
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
                    res.append(&mut Op::h(vec[i], 0).0);
                    res.append(
                        &mut Op::phi(
                            ((i+1)..count).map(|j| (vec[j], PI * (0.5 as R).pow((j-i) as u8)) ).collect(),
                            vec[i]).0
                    );
                }

                res.append(&mut Op::h(vec[count-1], 0).0);
                Op(res)
            }
        }
    }
    pub fn bench_circuit() -> Self {
        Op::id()
            * Op::h( 0b111, 0 )
            * Op::x( 0b010, 0b001 )
            * Op::h( 0b100, 0b001 )
            * Op::phi( vec![ (0b010, 1.) ], 0b001 )
            * Op::h( 0b001, 0b100 )
            * Op::z( 0b010, 0 )
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