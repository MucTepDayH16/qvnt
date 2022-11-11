//! Module contains definitions for quantum and classical registers.
//!
//! QVNT provide 3 types of registers:
//! * [`QReg`] - quantum register;
//! * [`CReg`] - classical register;
//! * [`VReg`] - *vurtual* register.

mod class;
mod quant;
mod virtl;

pub use class::Reg as CReg;
pub use quant::Reg as QReg;
pub use virtl::Reg as VReg;
