#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

#[cfg(test)]
mod tests;

mod types;
mod math;
mod bits_iter;

pub mod prelude {
    pub use super::operator::{applicable::Applicable, MultiOp, op};
    pub use super::register::{VReg, QReg};

    pub mod consts {
        pub const _1: crate::types::C = crate::types::C{ re: 1.0, im: 0.0 };
        pub const _0: crate::types::C = crate::types::C{ re: 0.0, im: 0.0 };
        pub const _i: crate::types::C = crate::types::C{ re: 0.0, im: 1.0 };

        pub const SQRT_1_2: crate::types::R = crate::types::SQRT_2 * 0.5;
    }
}

pub use crate::threads::num_threads;

pub mod operator;
pub mod register;
pub mod threads;