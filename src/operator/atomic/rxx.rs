use super::*;

#[derive(Clone, Copy, PartialEq)]
pub struct Op {
    ab_mask: Mask,
    phase: C,
}

impl Op {
    #[inline(always)]
    pub fn new(ab_mask: Mask, mut phase: R) -> Self {
        phase *= 0.5;
        let phase = C::new(phase.cos(), phase.sin());
        Self { ab_mask, phase }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    #[inline(always)]
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        let psi = (psi[idx], psi[idx ^ self.ab_mask]);
        C {
            re: psi.0.re * self.phase.re + psi.1.im * self.phase.im,
            im: psi.0.im * self.phase.re - psi.1.re * self.phase.im,
        }
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("RXX{}({})", self.ab_mask, 2.0 * self.phase.arg())
    }

    fn is_valid(&self) -> bool {
        self.ab_mask.count_ones() == 2
    }

    fn acts_on(&self) -> Mask {
        self.ab_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::RXX(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::RXX(Self {
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
    let cos = C {
        re: (0.5 * ANGLE).cos(),
        im: 0.0,
    };
    let i_sin = C {
        re: 0.0,
        im: (0.5 * ANGLE).sin(),
    };

    let op: SingleOp = Op::new(0b11, ANGLE).into();
    assert_eq!(op.name(), "RXX3(1.23456)");
    assert_eq!(
        op.matrix(2),
        [
            [cos, O, O, -i_sin],
            [O, cos, -i_sin, O],
            [O, -i_sin, cos, O],
            [-i_sin, O, O, cos]
        ]
    );
}
