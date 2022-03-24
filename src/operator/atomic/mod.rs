use std::{boxed::Box as Ptr};
use std::ops::Deref;
use super::single::SingleOp;
use crate::math::{C, R, N};

pub (crate) mod id;

pub (crate) mod x;
pub (crate) mod rx;
pub (crate) mod rxx;

pub (crate) mod y;
pub (crate) mod ry;
pub (crate) mod ryy;

pub (crate) mod z;
pub (crate) mod s;
pub (crate) mod t;
pub (crate) mod rz;
pub (crate) mod rzz;

pub (crate) mod phi;

pub (crate) mod h1;
pub (crate) mod h2;

pub (crate) mod swap;
pub (crate) mod i_swap;
pub (crate) mod sqrt_swap;
pub (crate) mod sqrt_i_swap;

pub (crate) mod dispatch;
pub use dispatch::*;