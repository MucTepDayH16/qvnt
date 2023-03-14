use std::{
    fmt,
    ops::{Mul, MulAssign},
};

use crate::math::types::*;

/// [`Classical register`](Reg)
///
/// Classical register represents the collapsed state of [`QReg`](super::QReg).
/// To make it simple, it is a typical binary number, like those in your computer.
/// But also, QVNT provide some interfaces, similar to [`QReg`](super::QReg):
///
/// * Constructor and initialization:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let q = QReg::with_state(8, 123);
/// let c = CReg::with_state(8, 123);
/// ```
///
/// * Tensor product of 2 cregs:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let c0 = CReg::with_state(4, 11);
/// let c1 = CReg::with_state(4, 7);
///
/// let c = c0 * c1;
/// # assert_eq!(c, CReg::with_state(8, 123));
/// ```
///
/// That make [`CReg`](Reg) like [`QReg`](super::QReg), but without superposition and entanglement.
///
/// You can obtain number from register using [`get()`](Reg::get):
///
/// ```rust
/// # use qvnt::prelude::*;
/// let c = CReg::with_state(8, 123);
///
/// // This will print 123
/// # assert_eq!(123, c.get());
/// println!("{}", c.get());
///
/// // Or Debug version
/// // which will prints specific bits
/// # assert_eq!("(01111011)", &format!("{:?}", c));
/// println!("{:?}", c);
/// ```
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Reg {
    value: N,
    q_num: N,
    q_mask: N,
}

impl Reg {
    /// Create classical register with a given number of bits.
    /// Initial value will be set to 0.
    pub fn new(q_num: N) -> Self {
        Self::with_state(q_num, 0)
    }

    /// Create classical register with a given number of bits
    /// and an initial state
    pub fn with_state(q_num: N, state: N) -> Self {
        let q_mask = 1_usize.wrapping_shl(q_num as u32).wrapping_sub(1_usize);

        Self {
            value: state,
            q_num,
            q_mask,
        }
    }

    pub fn num(&self) -> N {
        self.q_num
    }

    pub fn set_num(&mut self, q_num: N) {
        self.q_num = q_num;
        self.q_mask = 1_usize.wrapping_shl(q_num as u32).wrapping_sub(1_usize);
    }

    pub(crate) fn reset(&mut self, i_state: N) {
        self.value = i_state & self.q_mask;
    }

    pub fn set(&mut self, bit: bool, mask: N) {
        if bit {
            self.value |= mask;
        } else {
            self.value &= !mask;
        }
    }

    pub fn xor(&mut self, bit: bool, mask: N) {
        if bit {
            self.value ^= mask;
        }
    }

    fn tensor_prod(self, other: Self) -> Self {
        let shift = (0u8, self.q_num as u8);
        let state = (self.value << shift.0) | (other.value << shift.1);
        Self::with_state(self.q_num + other.q_num, state)
    }

    /// Obtain value from classing register.
    /// This number will always be less than 2<sup>N</sup>, where N is the number of bits, given in [`CReg::new()`](Reg::new).
    pub fn get(&self) -> N {
        self.value
    }

    pub(crate) fn get_by_mask(&self, mask: N) -> N {
        crate::math::bits_iter::BitsIter::from(mask & self.q_mask)
            .enumerate()
            .fold(0, |mask, (idx, val)| {
                if self.value & val != 0 {
                    mask | (1usize << idx)
                } else {
                    mask
                }
            })
    }
}

impl fmt::Debug for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value =
            crate::math::bits_iter::BitsIter::from(self.q_mask).fold(String::new(), |s, i| {
                if i & self.value == 0 {
                    format!("0{}", s)
                } else {
                    format!("1{}", s)
                }
            });
        write!(f, "({})", value)
    }
}

impl Mul for Reg {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.tensor_prod(other)
    }
}

impl MulAssign for Reg {
    fn mul_assign(&mut self, rhs: Self) {
        *self = std::mem::take(self).tensor_prod(rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let c = Reg::with_state(17, 123);

        println!("{:?}", c);
    }
}
