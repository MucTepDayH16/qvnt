use super::*;

#[derive(Clone, Copy, PartialEq)]
pub struct Op {
    a_mask: Mask,
    phase: C,
}

impl Op {
    #[inline(always)]
    pub fn new(a_mask: Mask, mut phase: R) -> Self {
        phase /= 2.;
        let phase = C::new(phase.cos(), phase.sin());
        Self { a_mask, phase }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    fn native_cpu_op(&self, psi: &[C], idx: N) -> C {
        let mut phase = self.phase;
        if idx & self.a_mask == 0 {
            phase.im = -phase.im;
        }
        phase * psi[idx]
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("RZ{}({})", self.a_mask, 2.0 * self.phase.arg())
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1
    }

    fn acts_on(&self) -> Mask {
        self.a_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::RZ(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::RZ(Self {
            phase: -self.phase,
            ..self
        })
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const ANGLE: R = 1.23456;

    const O: C = C { re: 0.0, im: 0.0 };
    let exp = C {
        re: (0.5 * ANGLE).cos(),
        im: (0.5 * ANGLE).sin(),
    };

    let op: SingleOp = Op::new(0b1, ANGLE).into();
    assert_eq!(op.name(), "RZ1(1.23456)");
    assert_eq!(op.matrix(1), [[exp.conj(), O], [O, exp]]);
}
