#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

pub use threads::num_threads;

#[doc = include_str ! ("../README.md")]
#[cfg(doctest)]
mod readme {}

#[cfg(test)]
mod tests;

mod math;

pub mod operator;
pub mod register;
pub mod threads;

pub mod prelude {
    pub use crate::{
        operator::{applicable::Applicable, MultiOp, op},
        register::{CReg, QReg, VReg},
    };

    pub mod consts {
        pub const _0: crate::math::C = crate::math::C_ZERO;
        pub const _1: crate::math::C = crate::math::C_ONE;
        pub const _i: crate::math::C = crate::math::C_IMAG;
        pub const SQRT_1_2: crate::math::C = crate::math::C { re: crate::math::FRAC_1_SQRT_2, im: 0.0 };
        pub const PI: crate::math::R = crate::math::PI;
    }
}