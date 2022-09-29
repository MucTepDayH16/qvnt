#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![doc = include_str!("../README.md")]

mod math;
#[cfg(feature = "cpu")]
mod threads;

pub mod operator;
pub mod register;

#[cfg(feature = "interpreter")]
pub mod qasm;

#[doc(hidden)]
pub mod prelude {
    #[cfg(feature = "interpreter")]
    pub use crate::qasm::{Ast, Int};
    pub use crate::{
        operator as op,
        operator::{Applicable, MultiOp, SingleOp},
        register::*,
    };
}
