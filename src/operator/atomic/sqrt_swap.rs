use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op {
    ab_mask: Mask,
    dagger: bool,
}

impl Op {
    #[inline(always)]
    pub fn new(ab_mask: Mask) -> Self {
        Self {
            ab_mask,
            dagger: false,
        }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    fn native_cpu_op(&self, psi: &[C], idx: N) -> C {
        if (idx & self.ab_mask).count_ones() & 1 == 1 {
            let psi = (psi[idx], psi[idx ^ self.ab_mask]);
            if self.dagger {
                C {
                    re: 0.5 * (psi.0.re + psi.0.im + psi.1.re - psi.1.im),
                    im: 0.5 * (psi.0.im - psi.0.re + psi.1.im + psi.1.re),
                }
            } else {
                C {
                    re: 0.5 * (psi.0.re - psi.0.im + psi.1.re + psi.1.im),
                    im: 0.5 * (psi.0.im + psi.0.re + psi.1.im - psi.1.re),
                }
            }
        } else {
            psi[idx]
        }
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("sqrt(SWAP{})", self.ab_mask)
    }

    fn is_valid(&self) -> bool {
        self.ab_mask.count_ones() == 2
    }

    fn acts_on(&self) -> Mask {
        self.ab_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::SqrtSwap(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::SqrtSwap(Self {
            dagger: !self.dagger,
            ..self
        })
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C { re: 0.0, im: 0.0 };
    const I: C = C { re: 1.0, im: 0.0 };
    const sqrt_i: C = C { re: 0.5, im: 0.5 };

    let op: SingleOp = Op::new(0b11).into();
    assert_eq!(op.name(), "sqrt(SWAP3)");
    assert_eq!(
        op.matrix(2),
        [
            [I, O, O, O],
            [O, sqrt_i, sqrt_i.conj(), O],
            [O, sqrt_i.conj(), sqrt_i, O],
            [O, O, O, I]
        ]
    );
}
