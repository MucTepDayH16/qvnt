use super::*;
use crate::math::matrix::{inverse_unitary_m1, is_unitary_m1};

const EXP_I_PI_4: C = C {
    re: crate::math::FRAC_1_SQRT_2,
    im: crate::math::FRAC_1_SQRT_2,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Op {
    a_mask: Mask,
    matrix: M1,
}

impl Op {
    pub fn new(a_mask: Mask, matrix: M1) -> Self {
        Self { a_mask, matrix }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    fn native_cpu_op(&self, psi: &[C], idx: N) -> C {
        if (idx & self.a_mask) == 0 {
            self.matrix[0b00] * psi[idx] + self.matrix[0b01] * psi[idx ^ self.a_mask]
        } else {
            self.matrix[0b10] * psi[idx ^ self.a_mask] + self.matrix[0b11] * psi[idx]
        }
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!(
            "U{}{:?}",
            self.a_mask,
            [&self.matrix[..2], &self.matrix[2..]]
        )
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1 && is_unitary_m1(&self.matrix)
    }

    fn acts_on(&self) -> Mask {
        self.a_mask
    }

    fn this(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::U1(self)
    }

    fn dgr(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::U1(Self {
            matrix: inverse_unitary_m1(&self.matrix),
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
    const SQRT_1_2: C = C {
        re: crate::math::FRAC_1_SQRT_2,
        im: 0.0,
    };

    let op = Op::new(0b1, [I, O, O, I]);
    assert!(op.is_valid());

    let op: SingleOp = op.into();
    assert_eq!(op.name(), format!("U1[[{I:?}, {O:?}], [{O:?}, {I:?}]]"));
    assert_eq!(op.matrix(1), [[I, O], [O, I]]);

    let op = Op::new(0b1, [I, I, O, I]);
    assert!(!op.is_valid());

    let op = Op::new(0b11, [I, O, O, I]);
    assert!(!op.is_valid());

    let op = Op::new(0b1, [SQRT_1_2, SQRT_1_2, SQRT_1_2, -SQRT_1_2]);
    assert!(op.is_valid());

    let op: SingleOp = op.into();
    assert_eq!(
        op.name(),
        format!(
            "U1[[{SQRT_1_2:?}, {SQRT_1_2:?}], [{SQRT_1_2:?}, {:?}]]",
            -SQRT_1_2
        )
    );
    assert_eq!(op.matrix(1), [[SQRT_1_2, SQRT_1_2], [SQRT_1_2, -SQRT_1_2]]);
}
