pub use std::f64::consts::*;

pub use self::{consts::*, types::*};

#[cfg(feature = "float-cmp")]
pub mod approx_cmp;
pub mod bits_iter;
#[cfg(feature = "float-cmp")]
pub mod matrix;

mod consts {
    use super::types::*;

    pub const C_ONE: C = C { re: 1., im: 0. };
    pub const C_ZERO: C = C { re: 0., im: 0. };
    pub const C_IMAG: C = C { re: 0., im: 1. };

    pub const I_POW_TABLE: [C; 4] = [
        C { re: 1., im: 0. },
        C { re: 0., im: 1. },
        C { re: -1., im: 0. },
        C { re: 0., im: -1. },
    ];
}

mod types {
    pub type N = usize;
    pub type Z = isize;

    pub type R = f64;
    pub type C = num_complex::Complex<R>;

    pub type M1 = [C; 4];
    pub type M2 = [C; 16];
}

#[inline]
pub fn count_bits(n: N) -> N {
    n.count_ones() as N
}

#[inline]
pub fn rotate(mut z: C, q: N) -> C {
    if q & 0b10 != 0 {
        z = -z;
    }
    if q & 0b01 != 0 {
        z.im = -z.im;
        std::mem::swap(&mut z.re, &mut z.im);
    }
    z
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
