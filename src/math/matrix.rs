use super::{approx_cmp::*, types::*};

pub fn is_diagonal_m1(u: &M1) -> bool {
    approx_cmp(u[0b01].norm_sqr(), 0.0) && approx_cmp(u[0b10].norm_sqr(), 0.0)
}

pub fn is_diagonal_m2(u: &M2) -> bool {
    approx_cmp(u[0b0001].norm_sqr(), 0.0)
        && approx_cmp(u[0b0010].norm_sqr(), 0.0)
        && approx_cmp(u[0b0011].norm_sqr(), 0.0)
        && approx_cmp(u[0b0100].norm_sqr(), 0.0)
        && approx_cmp(u[0b0110].norm_sqr(), 0.0)
        && approx_cmp(u[0b0111].norm_sqr(), 0.0)
        && approx_cmp(u[0b1000].norm_sqr(), 0.0)
        && approx_cmp(u[0b1001].norm_sqr(), 0.0)
        && approx_cmp(u[0b1011].norm_sqr(), 0.0)
        && approx_cmp(u[0b1100].norm_sqr(), 0.0)
        && approx_cmp(u[0b1101].norm_sqr(), 0.0)
        && approx_cmp(u[0b1110].norm_sqr(), 0.0)
}

pub fn is_unitary_m1(u: &M1) -> bool {
    let e00 = u[0b00].norm_sqr() + u[0b01].norm_sqr();
    let e11 = u[0b10].norm_sqr() + u[0b11].norm_sqr();
    let e01 = u[0b00] * u[0b10].conj() + u[0b01] * u[0b11].conj();

    approx_cmp(e00, 1.0)
        && approx_cmp(e11, 1.0)
        && approx_cmp(e01.re + e01.im, 0.0)
}

fn hermitian_mul(i: N, j: N, u: &M2) -> C {
    let i = (i << 2) & 0xf;
    let j = (j << 2) & 0xf;
    if i == j {
        C::new(
            (u[i].norm_sqr() + u[0b01 | i].norm_sqr())
                + (u[0b10 | i].norm_sqr() + u[0b11 | i].norm_sqr()),
            0.0,
        )
    } else {
        (u[i] * u[j].conj() + u[0b01 | i] * u[0b01 | j].conj())
            + (u[0b10 | i] * u[0b10 | j].conj()
                + u[0b11 | i] * u[0b11 | j].conj())
    }
}

pub fn is_unitary_m2(u: &M2) -> bool {
    let e00 = hermitian_mul(0, 0, u).re;
    let e11 = hermitian_mul(1, 1, u).re;
    let e22 = hermitian_mul(2, 2, u).re;
    let e33 = hermitian_mul(3, 3, u).re;
    let e01 = hermitian_mul(0, 1, u);
    let e02 = hermitian_mul(0, 2, u);
    let e03 = hermitian_mul(0, 3, u);
    let e12 = hermitian_mul(1, 2, u);
    let e13 = hermitian_mul(1, 3, u);
    let e23 = hermitian_mul(2, 3, u);

    approx_cmp(e00, 1.0)
        && approx_cmp(e11, 1.0)
        && approx_cmp(e22, 1.0)
        && approx_cmp(e33, 1.0)
        && approx_cmp(e01.re + e01.im, 0.0)
        && approx_cmp(e02.re + e02.im, 0.0)
        && approx_cmp(e03.re + e03.im, 0.0)
        && approx_cmp(e12.re + e12.im, 0.0)
        && approx_cmp(e13.re + e13.im, 0.0)
        && approx_cmp(e23.re + e23.im, 0.0)
}

pub fn is_scaled_unitary_m1(u: &M1) -> bool {
    let e00 = u[0b00].norm_sqr() + u[0b01].norm_sqr();
    let e11 = u[0b10].norm_sqr() + u[0b11].norm_sqr();
    let e01 = u[0b00] * u[0b10].conj() + u[0b01] * u[0b11].conj();

    approx_cmp(e00, e11) && approx_cmp(e01.re + e01.im, 0.0)
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
