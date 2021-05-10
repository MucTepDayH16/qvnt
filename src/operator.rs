use {
    std::{
        collections::{
            BTreeMap,
            VecDeque,
        },
        boxed::Box,
        fmt,
        ops::{
            BitOr,
            BitOrAssign,
        },
        string::String
    },

    crate::{
        math::*,
    }
};

pub struct Op {
    pub(crate) name: String,
    pub(crate) func: Option<Box<dyn Fn(&Vec<C>, N) -> C + Send + Sync>>,
}

impl Op {
    pub fn id() -> Self {
        Self{
            name: String::from( "Id" ),
            func: None,
        }
    }

    pub fn x( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        Self {
            name: format!("X_a{}", a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    psi[idx ^ a_mask]
            )),
        }
    }

    pub fn cx( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::x(a_mask); }

        Self {
            name: format!("X_c{}_a{}", c_mask, a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    psi[if !idx & c_mask == 0 { idx ^ a_mask } else { idx }]
            )),
        }
    }

    pub fn y( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        let i = I_POW_TABLE[ (!count_bits(a_mask)).wrapping_add(1) & 0x3 ];

        Self {
            name: format!("X_a{}", a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    i * if count_bits(idx & a_mask).is_odd() {
                        -psi[idx ^ a_mask]
                    } else {
                        psi[idx ^ a_mask]
                    }
            )),
        }
    }

    pub fn cy( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::y(a_mask); }

        let i = I_POW_TABLE[ (!count_bits(a_mask)).wrapping_add(1) & 0x3 ];

        Self {
            name: format!("X_c{}_a{}", c_mask, a_mask),
            func: Some(Box::new(
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
            )),
        }
    }

    pub fn z( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        Self {
            name: format!("Z_a{}", a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    if count_bits(idx & a_mask).is_odd() {
                        -psi[idx]
                    } else {
                        psi[idx]
                    }
            ))
        }
    }

    pub fn cz( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::z(a_mask); }

        Self {
            name: format!("Z_c{}_a{}", c_mask, a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    if !idx & c_mask == 0 && count_bits(idx & a_mask).is_odd() {
                        -psi[idx]
                    } else {
                        psi[idx]
                    }
            )),
        }
    }

    pub fn phi( angles_vec: Vec<(N, R)>, c_mask: N ) -> Self {
        let angles = angles_vec.clone()
            .into_iter()
            .filter_map(
                |(jdx, ang)|
                    if c_mask & jdx != 0 {
                        None
                    } else {
                        let phase = phase_from_rad(ang);
                        if phase == ANGLE_TABLE[ 0 ] {
                            None
                        } else {
                            Some((jdx, phase))
                        }
                    }
            ).collect::<BTreeMap<N, C>>();

        if angles.is_empty() { return Self::id(); }

        Self {
            name: if c_mask == 0 {
                format!("RZ_{:?}", angles_vec)
            } else {
                format!("RZ_c{}_{:?}", c_mask, angles_vec)
            },
            func: Some(Box::new(
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
            )),
        }
    }

    pub fn h( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        let sqrt = {
            let sqrt_count = a_mask.count_ones() as N;
            let mut sqrt = R::one();
            if sqrt_count.is_odd() {
                sqrt *= SQRT_2 * 0.5;
            }
            sqrt / (N::one() << (sqrt_count >> 1)) as R
        };

        Self {
            name: format!("H_a{}", a_mask),
            func: Some(Box::new(
                move |psi, idy|
                    (0..psi.len()).filter_map(
                        |idx|
                            if (idx ^ idy) & !a_mask == 0 {
                                Some(if (idx & idy & a_mask).count_ones().is_even() {
                                    psi[idx]
                                } else {
                                    -psi[idx]
                                })
                            } else {
                                None
                            }
                    ).sum::<C>() * sqrt
            )),
        }
    }

    pub fn ch( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::h( a_mask ); }

        let sqrt = {
            let sqrt_count = a_mask.count_ones() as N;
            let mut sqrt = R::one();
            if sqrt_count.is_odd() {
                sqrt *= SQRT_2 * 0.5;
            }
            sqrt / (N::one() << (sqrt_count >> 1)) as R
        };

        Self {
            name: format!("H_c{}_a{}", c_mask, a_mask),
            func: Some(Box::new(
                move |psi, idy|
                    if !idy & c_mask == 0 {
                        (0..psi.len()).filter_map(
                            |idx|
                                if (idx ^ idy) & !a_mask == 0 {
                                    Some(if (idx & idy & a_mask).count_ones().is_even() {
                                        psi[idx]
                                    } else {
                                        -psi[idx]
                                    })
                                } else {
                                    None
                                }
                        ).sum::<C>() * sqrt
                    } else {
                        psi[idy]
                    }
            )),
        }
    }

    pub fn qft_no_swap( a_mask: N ) -> PendingOps {
        let count = a_mask.count_ones() as usize;
        match count {
            0 => PendingOps::new(),
            1 => PendingOps::new() | Self::h(a_mask),
            _ => {
                let mut pend_ops = PendingOps::new();
                let mut vec = Vec::<usize>::with_capacity(count);

                for idx in 0..64 {
                    let jdx = 1 << idx;
                    if jdx & a_mask != 0 {
                        vec.push(jdx);
                    }
                }

                for i in 0..(count-1) {
                    pend_ops = pend_ops
                        | Op::h(vec[i])
                        | Op::phi(((i+1)..count).map(|j| (vec[j], PI * (0.5 as R).pow((j-i) as u8)) ).collect(), vec[i]);
                }

                pend_ops | Op::h(vec[count-1])
            }
        }
    }
}

impl fmt::Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl Default for Op {
    fn default() -> Self {
        Self::id()
    }
}

#[derive(Debug)]
pub struct PendingOps(pub(crate) VecDeque<Op>);

impl PendingOps {
    pub fn new() -> Self {
        Self(VecDeque::with_capacity(32))
    }

    pub fn len(&self) -> N {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl Default for PendingOps {
    fn default() -> Self {
        Self::new()
    }
}

impl BitOr<Op> for PendingOps {
    type Output = Self;

    #[inline]
    fn bitor(mut self, rhs: Op) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl BitOrAssign<Op> for PendingOps {
    #[inline]
    fn bitor_assign(&mut self, rhs: Op) {
        self.0.push_back(rhs)
    }
}

impl BitOr for PendingOps {
    type Output = Self;

    #[inline]
    fn bitor(mut self, rhs: Self) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl BitOrAssign for PendingOps {
    #[inline]
    fn bitor_assign(&mut self, mut rhs: Self) {
        self.0.append(&mut rhs.0)
    }
}

impl<'a> BitOr<Op> for &'a mut PendingOps {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Op) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl<'a> BitOrAssign<Op> for &'a mut PendingOps {
    #[inline]
    fn bitor_assign(&mut self, rhs: Op) {
        self.0.push_back(rhs)
    }
}

impl<'a> BitOr<PendingOps> for &'a mut PendingOps {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: PendingOps) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl<'a> BitOrAssign<PendingOps> for &'a mut PendingOps {
    fn bitor_assign(&mut self, mut rhs: PendingOps) {
        self.0.append(&mut rhs.0)
    }
}