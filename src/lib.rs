#![allow(dead_code)]
#![warn(clippy::cargo)]
#![doc = include_str!("../README.md")]

mod math;
#[cfg(feature = "multi-thread")]
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
