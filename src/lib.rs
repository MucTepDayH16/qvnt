#![allow(unused_imports)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
mod README {}

mod math;

pub mod operator;
pub mod register;

#[doc(hidden)]
pub mod threads;
#[doc(hidden)]
pub use threads::num_threads;

pub mod prelude {
    pub use crate::{
        operator as op,
        operator::{MultiOp, SingleOp, Applicable},
        register::*,
    };
}