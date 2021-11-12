//! Module contains definitions for quantum and classical registers.
//!
//! QVNT provide 3 types of registers:
//! * [`QReg`] - quantum register;
//! * [`CReg`] - classical register;
//! * [`VReg`] - *vurtual* register.
//!

pub (crate) mod virtl;
pub (crate) mod quant;
pub (crate) mod class;

pub use quant::Reg as QReg;
pub use class::Reg as CReg;
pub use virtl::Reg as VReg;