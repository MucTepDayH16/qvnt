use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op {
    ab_mask: N,
    dagger: bool,
}

impl Op {
    #[inline(always)]
    pub fn new(ab_mask: N) -> Self {
        Self {
            ab_mask,
            dagger: false,
        }
    }
}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        if (idx & self.ab_mask).count_ones() & 1 == 1 {
            let psi = (psi[idx], psi[idx ^ self.ab_mask]);
            if self.dagger {
                C {
                    re: FRAC_1_SQRT_2 * (psi.0.re + psi.1.im),
                    im: FRAC_1_SQRT_2 * (psi.0.im - psi.1.re),
                }
            } else {
                C {
                    re: FRAC_1_SQRT_2 * (psi.0.re - psi.1.im),
                    im: FRAC_1_SQRT_2 * (psi.0.im + psi.1.re),
                }
            }
        } else {
            psi[idx]
        }
    }

    fn name(&self) -> String {
        format!("sqrt(iSWAP{})", self.ab_mask)
    }

    fn is_valid(&self) -> bool {
        self.ab_mask.count_ones() == 2
    }

    fn acts_on(&self) -> N {
        self.ab_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::SqrtISwap(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::SqrtISwap(Self {
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
    const i: C = C { re: 0.0, im: 1.0 };

    let op: SingleOp = Op::new(0b11).into();
    assert_eq!(op.name(), "sqrt(iSWAP3)");
    assert_eq!(
        op.matrix(2),
        [
            [I, O, O, O],
            [O, FRAC_1_SQRT_2 * I, FRAC_1_SQRT_2 * i, O],
            [O, FRAC_1_SQRT_2 * i, FRAC_1_SQRT_2 * I, O],
            [O, O, O, I]
        ]
    );
}
