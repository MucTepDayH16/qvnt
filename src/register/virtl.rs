use std::{
    cell::RefCell,
    ops::{Index, RangeFull},
    rc::Rc,
};

use crate::math::{bits_iter, C, N, R};

type Ptr<T> = Rc<RefCell<T>>;

/// [`Virtual register`](Reg)
///
/// Virtual registers have only one purpose: to *humanify* interaction with qubits and gates.
/// Instead of using bitmask as argumants for gates or [`QReg::measure(...)`](super::QReg::measure())
/// you are able to use [`VReg`](Reg) instead. E.g., using bitmask notation:
///
/// ```rust
/// # use qvnt::prelude::*;
///
/// let mut q = QReg::new(8).init_state(0b00111000);
/// let gate = op::h(0b01010101) * op::x(0b10101010);
///
/// q.apply(&gate);
///
/// // this is equivalent to just q.measure()
/// // but e.g.
/// let c = q.measure_mask(0b11111111);
/// println!("{}", c.get());
/// ```
///
/// could be very confusing.
/// Moreover, it will became more complicated to construct mask for a bigger amount of qubits.
/// So, we could rewrite this example using [`VReg`](Reg):
///
/// ```rust
/// # use qvnt::prelude::*;
/// fn even(i: usize) -> bool {
///     i % 2 == 0
/// }
///
/// // Create virtual register with same amount of qubits
/// let v = VReg::new(8);
/// // mask 0b00011000 acts on qubits 4, 5 and 6, so...
/// # assert_eq!(v[[3, 4, 5]], 0b00111000);
/// let mut q = QReg::new(8).init_state(v[[3, 4, 5]]);
/// // indexing with fn
/// # assert_eq!(v[even], 0b01010101);
/// # assert_eq!(v[|i| i % 2 != 0], 0b10101010);
/// let gate = op::h(v[even])
/// // indexing with closure
///     * op::h(v[|i| i % 2 != 0]);
///
/// q.apply(&gate);
///
/// # assert_eq!(v[..], 0b11111111);
/// let c = q.measure_mask(v[..]);
/// println!("{}", c.get());
/// ```
///
/// You are also able to create [`VReg`](Reg) from existing quantum register:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let q = QReg::new(10);
/// let v = q.get_vreg();
/// ```
///
/// Or specify different qubits by aliases:
///
/// ```rust
/// # use qvnt::prelude::*;
/// let v = VReg::new(8);
/// // e represents all even qubits of register q
/// let e = VReg::from(v[|i| i % 2 == 0]);
/// // o represents all odd qubits of register q
/// let o = VReg::from(v[|i| i % 2 != 0]);
/// # assert_eq!(e[..], 0b01010101);
/// # assert_eq!(o[..], 0b10101010);
/// let gate = op::h(e[..]) * op::x(o[..]);
///
/// let mut q = QReg::new(8);
/// q.apply(&gate);
/// ```
#[derive(Clone, Default)]
pub struct Reg(pub(crate) Ptr<N>, pub(crate) Vec<N>);

impl Reg {
    /// Create virtual register with a given number of qubits.
    pub fn new(num: N) -> Self {
        Self::new_with_mask(
            1usize.wrapping_shl(num as u32).wrapping_add(!0usize),
        )
    }

    pub(crate) fn new_with_mask(mask: N) -> Self {
        let bi = bits_iter::BitsIter::from(mask);
        super::VReg(Ptr::new(0.into()), bi.collect())
    }
}

impl From<N> for Reg {
    fn from(mask: N) -> Self {
        Self::new_with_mask(mask)
    }
}

impl std::fmt::Debug for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x?}", self.1)
    }
}

impl Index<N> for Reg {
    type Output = N;

    #[inline]
    fn index(&self, idx: N) -> &Self::Output {
        &self.1[idx]
    }
}

impl<F> Index<F> for Reg
where
    F: Fn(N) -> bool,
{
    type Output = N;

    #[inline]
    fn index(&self, f: F) -> &Self::Output {
        let tmp = self
            .1
            .iter()
            .enumerate()
            .filter_map(|(i, j)| if f(i) { Some(*j) } else { None })
            .fold(0, |acc, idx| acc | idx);
        self.0.replace(tmp);
        unsafe { self.0.as_ptr().as_ref().unwrap() }
    }
}

impl<const X: usize> Index<[N; X]> for Reg {
    type Output = N;

    fn index(&self, slice: [N; X]) -> &Self::Output {
        &self[move |i| slice.contains(&i)]
    }
}

impl Index<RangeFull> for Reg {
    type Output = N;

    #[inline]
    fn index(&self, _: RangeFull) -> &Self::Output {
        fn all(_: N) -> bool {
            true
        }
        self.index(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::single_thread::SingleThread;

    #[test]
    fn index() {
        fn even(i: N) -> bool {
            i % 2 == 0
        }

        fn odd(i: N) -> bool {
            i % 2 != 0
        }

        let v = Reg::new(8);
        assert_eq!(v[3], 0b00001000);
        assert_eq!(v[even], 0b01010101);
        assert_eq!(v[odd], 0b10101010);
        assert_eq!(v[..], 0b11111111);
        assert_eq!(v[|_| true], 0b11111111);
        assert_eq!(v[[0, 7]], 0b10000001);
    }

    #[test]
    fn virtual_regs() {
        use crate::prelude::*;

        let mut reg = QReg::new(0);

        //  qreg    x[3];
        reg *= QReg::new(3);
        //  qreg    a[2];
        reg *= QReg::new(2);

        let r = reg.get_vreg();
        let m = reg.get_vreg_by(0b01101).unwrap();
        let x = reg.get_vreg_by(0b00111).unwrap();
        let y = reg.get_vreg_by(0b11000).unwrap();

        assert_eq!(r[0], 0b00001);
        assert_eq!(r[1], 0b00010);
        assert_eq!(r[2], 0b00100);
        assert_eq!(r[3], 0b01000);
        assert_eq!(r[4], 0b10000);
        assert_eq!(r[..], 0b11111);

        assert_eq!(m[0], 0b00001);
        assert_eq!(m[1], 0b00100);
        assert_eq!(m[2], 0b01000);
        assert_eq!(m[..], 0b01101);

        assert_eq!(x[0], 0b00001);
        assert_eq!(x[1], 0b00010);
        assert_eq!(x[2], 0b00100);
        assert_eq!(x[..], 0b00111);

        assert_eq!(y[0], 0b01000);
        assert_eq!(y[1], 0b10000);
        assert_eq!(y[..], 0b11000);
    }
}
