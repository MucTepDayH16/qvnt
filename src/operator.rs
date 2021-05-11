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

    crate::{
        math::*,
    }
};

pub(crate) struct Operator {
    pub(crate) name: String,
    pub(crate) func: Box<dyn Fn(&Vec<C>, N) -> C + Send + Sync>,
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
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

    #[inline]
    fn new_operator(name: String, func: Box<dyn Fn(&Vec<C>, N) -> C + Send + Sync>) -> Self {
        Self(VecDeque::from(vec![Operator{name, func}]))
    }

    #[inline]
    fn from_operatot(op: Operator) -> Self {
        Self(VecDeque::from(vec![op]))
    }

    pub fn id() -> Self {
        Self(VecDeque::new())
    }

    pub fn x( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        Self::new_operator(
            format!("X_a{}", a_mask),
            Box::new(
                move |psi, idx|
                    psi[idx ^ a_mask]
            )
        )
    }

    pub fn cx( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::x(a_mask); }

        Self::new_operator(
            format!("X_c{}_a{}", c_mask, a_mask),
            Box::new(
                move |psi, idx|
                    psi[if !idx & c_mask == 0 { idx ^ a_mask } else { idx }]
            )
        )
    }

    pub fn y( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        let i = I_POW_TABLE[(!count_bits(a_mask)).wrapping_add(1) & 0x3];

        Self::new_operator(
            format!("X_a{}", a_mask),
            Box::new(
                move |psi, idx|
                    i * if count_bits(idx & a_mask).is_odd() {
                        -psi[idx ^ a_mask]
                    } else {
                        psi[idx ^ a_mask]
                    }
            )
        )
    }

    pub fn cy( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::y(a_mask); }

        let i = I_POW_TABLE[ (!count_bits(a_mask)).wrapping_add(1) & 0x3 ];

        Self::new_operator(
            format!("X_c{}_a{}", c_mask, a_mask),
            Box::new(
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
        )
    }

    pub fn z( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        Self::new_operator(
            format!("Z_a{}", a_mask),
            Box::new(
                move |psi, idx|
                    if count_bits(idx & a_mask).is_odd() {
                        -psi[idx]
                    } else {
                        psi[idx]
                    }
            )
        )
    }

    pub fn cz( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::z(a_mask); }

        Self::new_operator(
            format!("Z_c{}_a{}", c_mask, a_mask),
            Box::new(
                move |psi, idx|
                    if !idx & c_mask == 0 && count_bits(idx & a_mask).is_odd() {
                        -psi[idx]
                    } else {
                        psi[idx]
                    }
            )
        )
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

        Self::new_operator(
            if c_mask == 0 {
                format!("RZ_{:?}", angles_vec)
            } else {
                format!("RZ_c{}_{:?}", c_mask, angles_vec)
            },
            Box::new(
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
        )
    }

    pub fn h( a_mask: N ) -> Self {
        fn h_op_1x1(a_mask: N) -> Operator {
            #[cfg(test)] {
                assert_eq!(a_mask.count_ones(), 1);
            }
            const SQRT_1_2: R = SQRT_2 * 0.5;

            Operator {
                name: format!("H_a{}", a_mask),
                func: Box::new(
                    move |psi, idx| {
                        let a = (idx & a_mask) != 0;
                        (   if a { -psi[idx] } else { psi[idx] }    +
                            psi[idx ^ a_mask]                       ) * SQRT_1_2
                    }
                )
            }
        }
        fn h_op_2x2(a_mask: N, b_mask: N) -> Operator {
            #[cfg(test)] {
                assert_eq!(a_mask.count_ones(), 1);
                assert_eq!(b_mask.count_ones(), 1);
                assert_eq!(a_mask & b_mask, 0);
            }

            let ab_mask = a_mask | b_mask;

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
        }

        let count = count_bits(a_mask);

        match count {
            0 => Self::id(),
            1 => Self::from_operatot(h_op_1x1(a_mask)),
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
                            res.0.push_back(h_op_2x2(idx.0, idx.1));
                            is_first = true;
                        }
                    }
                    idx.0 <<= 1;
                }

                if !is_first {
                    res.0.push_back(h_op_1x1(idx.1));
                }

                res
            }
        }
    }

    pub fn ch( a_mask: N, c_mask: N ) -> Self {
        fn ch_op_1x1(a_mask: N, c_mask: N) -> Operator {
            #[cfg(test)] {
                assert_eq!(a_mask.count_ones(), 1);

                assert_eq!(a_mask & c_mask, 0);
            }
            const SQRT_1_2: R = SQRT_2 * 0.5;

            Operator {
                name: format!("H_a{}", a_mask),
                func: Box::new(
                    move |psi, idx|
                        if !idx & c_mask == 0 {
                            let a = (idx & a_mask) != 0;
                            (   if a { -psi[idx] } else { psi[idx] }    +
                                psi[idx ^ a_mask]                       ) * SQRT_1_2
                        } else {
                            psi[idx]
                        }
                )
            }
        }
        fn ch_op_2x2(a_mask: N, b_mask: N, c_mask: N) -> Operator {
            #[cfg(test)] {
                assert_eq!(a_mask.count_ones(), 1);
                assert_eq!(b_mask.count_ones(), 1);
                assert_eq!(a_mask & b_mask, 0);

                assert_eq!(a_mask & c_mask, 0);
                assert_eq!(b_mask & c_mask, 0);
            }

            let ab_mask = a_mask | b_mask;

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

        let a_mask = a_mask & !c_mask;
        let count = count_bits(a_mask);

        match count {
            0 => Self::id(),
            1 => Self::from_operatot(ch_op_1x1(a_mask, c_mask)),
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
                            res.0.push_back(ch_op_2x2(idx.0, idx.1, c_mask));
                            is_first = true;
                        }
                    }
                    idx.0 <<= 1;
                }

                if !is_first {
                    res.0.push_back(ch_op_1x1(idx.1, c_mask));
                }

                res
            }
        }
    }

    pub(crate) fn u_1x1(a_mask: N, matrix: [[C; 2]; 2]) -> Operator {
        let u = [matrix[0][0], matrix[0][1], matrix[1][0], matrix[1][1]];
        #[cfg(test)] {
            assert_eq!(a_mask.count_ones(), 1);

            assert!(is_unitary(u[0], u[1], u[2], u[3]));
        }

        if is_diagonal(u[0], u[1], u[2], u[3]) {
            Operator {
                name: format!("Diag[{:?}, {:?}]", u[0], u[3]),
                func: Box::new(
                    move |psi, idx|
                        u[if (idx & a_mask) != 0 { 3 } else { 0 }] * psi[idx]
                )
            }
        } else {
            Operator {
                name: format!("Unitary[{:?}, {:?}, {:?}, {:?}]", u[0], u[1], u[2], u[3]),
                func: Box::new(
                    move |psi, idx| {
                        let a = (idx & a_mask) != 0;
                        let udx = (if a { 3 } else { 0 }, if a { 2 } else { 1 });
                        u[udx.0] * psi[idx] + u[udx.1] * psi[idx ^ a_mask]
                    }
                )
            }
        }
    }

    pub fn qft_no_swap( a_mask: N ) -> Op {
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
                            ((i+1)..count).map(|j| (vec[j], PI * (0.5 as R).pow((j-i) as u8)) ).collect(),
                            vec[i]).0
                    );
                }

                res.append(&mut Op::h(vec[count-1]).0);
                Op(res)
            }
        }
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