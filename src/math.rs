pub use {
    float_cmp::*,
    crate::types::*,
};

pub const C_ONE: C = C{ re: 1., im: 0. };
pub const C_ZERO: C = C{ re: 0., im: 0. };

pub const I_POW_TABLE: [C; 4] = [
    C{ re: 1., im: 0. },
    C{ re: 0., im: 1. },
    C{ re: -1., im: 0. },
    C{ re: 0., im: -1. },
];

pub const ANGLE_TABLE: [C; 46] = [
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

#[inline]
pub fn count_bits(n: N) -> N {
    n.count_ones() as N
}

#[inline]
pub fn phase_from_rad(rad: R) -> C {
    /*
    let deg = (rad.to_degrees().round() as Z).mod_floor(&360) as N;
    let (quat, deg) = deg.div_mod_floor(&90);

    I_POW_TABLE[quat] *
        if deg > 45 {
            let c = ANGLE_TABLE[90 - deg];
            C { re: c.im, im: c.re }
        } else {
            ANGLE_TABLE[deg]
        }
     */
    C::from_polar(1.0, rad)
}

const ULPS: i64 = 2;

#[inline]
fn approx_real(x: &C) -> bool {
    approx_eq!(R, x.im, 0.0, ulps = ULPS)
}

#[inline]
fn approx_eq(a: &C, b: &C) -> bool {
    approx_eq!(R, a.re, b.re, ulps = ULPS) && approx_eq!(R, a.im, b.im, ulps = ULPS)
}

#[inline]
fn approx_eq_conj(a: &C, b: &C) -> bool {
    approx_eq!(R, a.re, b.re, ulps = ULPS) && approx_eq!(R, a.im, -b.im, ulps = ULPS)
}

pub fn is_diagonal_m1(u: &M1) -> bool {
    u[0b01].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b10].norm_sqr().approx_eq_ulps(&0.0, ULPS)
}

pub fn is_diagonal_m2(u: &M2) -> bool {
    u[0b0001].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b0010].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b0011].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b0100].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b0110].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b0111].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b1000].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b1001].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b1011].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b1100].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b1101].norm_sqr().approx_eq_ulps(&0.0, ULPS)
        && u[0b1110].norm_sqr().approx_eq_ulps(&0.0, ULPS)
}

pub fn is_unitary_m1(u: &M1) -> bool {
    let e00 = u[0b00].norm_sqr() + u[0b01].norm_sqr();
    let e11 = u[0b10].norm_sqr() + u[0b11].norm_sqr();
    let e01 = u[0b00]*u[0b10].conj() + u[0b01]*u[0b11].conj();

    e00.approx_eq_ulps(&1.0, ULPS)
        && e11.approx_eq_ulps(&1.0, ULPS)
        && (e01.re + e01.im).approx_eq_ulps(&0.0, ULPS)
}

fn m2_mul_m2_herm(i: N, j: N, u: &M2) -> C {
    let i = (i << 2) & 0xf;
    let j = (j << 2) & 0xf;
    if i == j {
        C::new((u[0b00 | i].norm_sqr() + u[0b01 | i].norm_sqr()) +
                   (u[0b10 | i].norm_sqr() + u[0b11 | i].norm_sqr()), 0.0)
    } else {
        (u[0b00 | i]*u[0b00 | j].conj() + u[0b01 | i]*u[0b01 | j].conj()) +
            (u[0b10 | i]*u[0b10 | j].conj() + u[0b11 | i]*u[0b11 | j].conj())
    }
}

pub fn is_unitary_m2(u: &M2) -> bool {
    let e00 = m2_mul_m2_herm(0, 0, u).re;
    let e11 = m2_mul_m2_herm(1, 1, u).re;
    let e22 = m2_mul_m2_herm(2, 2, u).re;
    let e33 = m2_mul_m2_herm(3, 3, u).re;
    let e01 = m2_mul_m2_herm(0, 1, u);
    let e02 = m2_mul_m2_herm(0, 2, u);
    let e03 = m2_mul_m2_herm(0, 3, u);
    let e12 = m2_mul_m2_herm(1, 2, u);
    let e13 = m2_mul_m2_herm(1, 3, u);
    let e23 = m2_mul_m2_herm(2, 3, u);

    e00.approx_eq_ulps(&1.0, ULPS)
        && e11.approx_eq_ulps(&1.0, ULPS)
        && e22.approx_eq_ulps(&1.0, ULPS)
        && e33.approx_eq_ulps(&1.0, ULPS)
        && (e01.re + e01.im).approx_eq_ulps(&0.0, ULPS)
        && (e02.re + e02.im).approx_eq_ulps(&0.0, ULPS)
        && (e03.re + e03.im).approx_eq_ulps(&0.0, ULPS)
        && (e12.re + e12.im).approx_eq_ulps(&0.0, ULPS)
        && (e13.re + e13.im).approx_eq_ulps(&0.0, ULPS)
        && (e23.re + e23.im).approx_eq_ulps(&0.0, ULPS)
}

pub fn is_scaled_unitary_m1(u: &M1) -> bool {
    let e00 = u[0b00].norm_sqr() + u[0b01].norm_sqr();
    let e11 = u[0b10].norm_sqr() + u[0b11].norm_sqr();
    let e01 = u[0b00]*u[0b10].conj() + u[0b01]*u[0b11].conj();

    e00.approx_eq_ulps(&e11, ULPS)
        && (e01.re + e01.im).approx_eq_ulps(&0.0, ULPS)
}

pub fn is_scaled_unitary_m2(_: &M2) -> bool {
    todo!()
}

pub fn is_hermitian_m1(u: &M1) -> bool {
    approx_real(&u[0b00])
        && approx_eq_conj(&u[0b01], &u[0b10])
        && approx_real(&u[0b11])
}

pub fn is_hermitian_m2(u: &M2) -> bool {
    approx_real(&u[0b0000])
        && approx_eq_conj(&u[0b0001], &u[0b0100])
        && approx_eq_conj(&u[0b0010], &u[0b1000])
        && approx_eq_conj(&u[0b0011], &u[0b1100])
        && approx_real(&u[0b0101])
        && approx_eq_conj(&u[0b0110], &u[0b1001])
        && approx_eq_conj(&u[0b0111], &u[0b1101])
        && approx_real(&u[0b1010])
        && approx_eq_conj(&u[0b1011], &u[0b1110])
        && approx_real(&u[0b1111])
}