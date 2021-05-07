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

    crate::types::{*, nums::*}
};
use std::mem::take;
use std::ops::BitOrAssign;

static I_POW_TABLE: [C; 4] = [
    C{ re: 1., im: 0. },
    C{ re: 0., im: 1. },
    C{ re: -1., im: 0. },
    C{ re: 0., im: -1. },
];

static ANGLE_TABLE: [C; 46] = [
    C{ re: 1.0000000000000000, im: 0.00000000000000000 },
    C{ re: 0.9998476951563913, im: 0.01745240643728351 },
    C{ re: 0.9993908270190958, im: 0.03489949670250097 },
    C{ re: 0.9986295347545738, im: 0.05233595624294383 },
    C{ re: 0.9975640502598242, im: 0.06975647374412530 },
    C{ re: 0.9961946980917455, im: 0.08715574274765817 },
    C{ re: 0.9945218953682733, im: 0.10452846326765346 },
    C{ re: 0.9925461516413220, im: 0.12186934340514748 },
    C{ re: 0.9902680687415704, im: 0.13917310096006544 },
    C{ re: 0.9876883405951378, im: 0.15643446504023087 },
    C{ re: 0.9848077530122080, im: 0.17364817766693033 },
    C{ re: 0.9816271834476640, im: 0.19080899537654480 },
    C{ re: 0.9781476007338057, im: 0.20791169081775931 },
    C{ re: 0.9743700647852352, im: 0.22495105434386500 },
    C{ re: 0.9702957262759965, im: 0.24192189559966773 },
    C{ re: 0.9659258262890683, im: 0.25881904510252074 },
    C{ re: 0.9612616959383189, im: 0.27563735581699916 },
    C{ re: 0.9563047559630354, im: 0.29237170472273677 },
    C{ re: 0.9510565162951535, im: 0.30901699437494740 },
    C{ re: 0.9455185755993168, im: 0.32556815445715664 },
    C{ re: 0.9396926207859084, im: 0.34202014332566870 },
    C{ re: 0.9335804264972017, im: 0.35836794954530027 },
    C{ re: 0.9271838545667874, im: 0.37460659341591200 },
    C{ re: 0.9205048534524404, im: 0.39073112848927370 },
    C{ re: 0.9135454576426009, im: 0.40673664307580015 },
    C{ re: 0.9063077870366499, im: 0.42261826174069944 },
    C{ re: 0.8987940462991670, im: 0.43837114678907740 },
    C{ re: 0.8910065241883679, im: 0.45399049973954675 },
    C{ re: 0.8829475928589270, im: 0.46947156278589080 },
    C{ re: 0.8746197071393957, im: 0.48480962024633706 },
    C{ re: 0.8660254037844387, im: 0.49999999999999994 },
    C{ re: 0.8571673007021123, im: 0.51503807491005420 },
    C{ re: 0.8480480961564260, im: 0.52991926423320490 },
    C{ re: 0.8386705679454240, im: 0.54463903501502710 },
    C{ re: 0.8290375725550417, im: 0.55919290347074690 },
    C{ re: 0.8191520442889918, im: 0.57357643635104600 },
    C{ re: 0.8090169943749475, im: 0.58778525229247310 },
    C{ re: 0.7986355100472928, im: 0.60181502315204830 },
    C{ re: 0.7880107536067220, im: 0.61566147532565820 },
    C{ re: 0.7771459614569709, im: 0.62932039104983740 },
    C{ re: 0.7660444431189780, im: 0.64278760968653930 },
    C{ re: 0.7547095802227720, im: 0.65605902899050720 },
    C{ re: 0.7431448254773942, im: 0.66913060635885820 },
    C{ re: 0.7313537016191706, im: 0.68199836006249850 },
    C{ re: 0.7193398003386512, im: 0.69465837045899730 },
    C{ re: 0.7071067811865476, im: 0.70710678118654760 },
];

pub struct Op {
    pub(crate) name: String,
    pub(crate) func: Option<Box<dyn Fn(&Vec<C>, N) -> C + Send + Sync>>,
}

impl Op {
    #[inline]
    fn phase_from_rad(rad: R) -> C {
        let deg = (rad.to_degrees().round() as Z).mod_floor(&360) as N;
        let (quat, deg) = deg.div_mod_floor(&90);

        I_POW_TABLE[quat] *
            if deg > 45 {
                let c = ANGLE_TABLE[90 - deg];
                C { re: c.im, im: c.re }
            } else {
                ANGLE_TABLE[deg]
            }
    }

    pub fn id() -> Self {
        Self{
            name: String::from( "Id" ),
            func: None,
        }
    }

    pub fn x( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::x_uc( a_mask ); }

        Self {
            name: format!("X_c{}_a{}", c_mask, a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    psi[if !idx & c_mask == 0 { idx ^ a_mask } else { idx }]
            )),
        }
    }

    pub fn x_uc( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        Self {
            name: format!("X_a{}", a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    psi[idx ^ a_mask]
            )),
        }
    }

    pub fn z( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::z_uc( a_mask ); }

        Self {
            name: format!("Z_c{}_a{}", c_mask, a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    if !idx & c_mask == 0 && (idx & a_mask).count_ones().is_odd() {
                        -psi[idx]
                    } else {
                        psi[idx]
                    }
            )),
        }
    }

    pub fn z_uc( a_mask: N ) -> Self {
        if a_mask == 0 { return Self::id(); }

        Self {
            name: format!("Z_a{}", a_mask),
            func: Some(Box::new(
                move |psi, idx|
                    if (idx & a_mask).count_ones().is_odd() {
                        -psi[idx]
                    } else {
                        psi[idx]
                    }
            ))
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
                        let phase = Self::phase_from_rad(ang);
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

    pub fn h( a_mask: N, c_mask: N ) -> Self {
        let a_mask = a_mask & !c_mask;
        if a_mask == 0 { return Self::id(); }
        if c_mask == 0 { return Self::h_uc( a_mask ); }

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
                        (0..psi.len()).into_iter().filter_map(
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

    pub fn h_uc( a_mask: N ) -> Self {
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

    pub fn qft_no_swap( a_mask: N ) -> PendingOps {
        let count = a_mask.count_ones() as usize;
        match count {
            0 => PendingOps::new(),
            1 => PendingOps::new() | Self::h_uc(a_mask),
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
                        | Op::h_uc(vec[i])
                        | Op::phi(((i+1)..count).map(|j| (vec[j], PI * (0.5 as R).pow((j-i) as u8)) ).collect(), vec[i]);
                }

                pend_ops | Op::h_uc(vec[count-1])
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