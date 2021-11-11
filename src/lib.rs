#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
mod README {}

mod math;

pub mod operator;
pub mod register;

#[cfg(feature = "cpu")]
#[doc(hidden)]
pub mod threads;
#[cfg(feature = "cpu")]
#[doc(hidden)]
pub use threads::num_threads;
#[cfg(not(feature = "cpu"))]
pub mod threads {
    pub (crate) fn global_install<OP, R>(op: OP) -> R
    where OP: FnOnce() -> R, { op() }
}

pub mod prelude {
    pub use crate::{
        operator as op,
        operator::{MultiOp, SingleOp, Applicable},
        register::*,
    };
}