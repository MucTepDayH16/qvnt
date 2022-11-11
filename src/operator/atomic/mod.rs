#![cfg_attr(test, allow(non_upper_case_globals))]

use crate::math::{consts::*, types::*};

pub mod id;

pub mod rx;
pub mod rxx;
pub mod x;

pub mod ry;
pub mod ryy;
pub mod y;

pub mod rz;
pub mod rzz;
pub mod s;
pub mod t;
pub mod z;

pub mod u1;
pub mod u2;

pub mod h1;
pub mod h2;

pub mod i_swap;
pub mod sqrt_i_swap;
pub mod sqrt_swap;
pub mod swap;

pub mod dispatch;
pub use self::dispatch::*;
