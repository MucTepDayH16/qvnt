use super::*;
use crate::math::matrix::{inverse_unitary_m2, is_unitary_m2};

#[derive(Clone, Copy, PartialEq)]
pub struct Op {
    a_mask: N,
    b_mask: N,
    matrix: M2,
}

impl Op {
    pub fn new(a_mask: N, b_mask: N, matrix: M2) -> Self {
        Self {
            a_mask,
            b_mask,
            matrix,
        }
    }
}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let a_bit = (idx & self.a_mask) != 0;
        let b_bit = (idx & self.b_mask) != 0;
        let idx = idx & !self.a_mask & !self.b_mask;
        if !b_bit {
            if !a_bit {
                self.matrix[0b0000] * psi[idx]
                    + self.matrix[0b0001] * psi[idx | self.a_mask]
                    + self.matrix[0b0010] * psi[idx | self.b_mask]
                    + self.matrix[0b0011] * psi[idx | self.a_mask | self.b_mask]
            } else {
                self.matrix[0b0100] * psi[idx]
                    + self.matrix[0b0101] * psi[idx | self.a_mask]
                    + self.matrix[0b0110] * psi[idx | self.b_mask]
                    + self.matrix[0b0111] * psi[idx | self.a_mask | self.b_mask]
            }
        } else if !a_bit {
            self.matrix[0b1000] * psi[idx]
                + self.matrix[0b1001] * psi[idx | self.a_mask]
                + self.matrix[0b1010] * psi[idx | self.b_mask]
                + self.matrix[0b1011] * psi[idx | self.a_mask | self.b_mask]
        } else {
            self.matrix[0b1100] * psi[idx]
                + self.matrix[0b1101] * psi[idx | self.a_mask]
                + self.matrix[0b1110] * psi[idx | self.b_mask]
                + self.matrix[0b1111] * psi[idx | self.a_mask | self.b_mask]
        }
    }

    fn name(&self) -> String {
        format!(
            "U{}{:?}",
            self.a_mask | self.b_mask,
            [
                &self.matrix[..4],
                &self.matrix[4..8],
                &self.matrix[8..12],
                &self.matrix[12..]
            ]
        )
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1
            && self.b_mask.count_ones() == 1
            && is_unitary_m2(&self.matrix)
    }

    fn acts_on(&self) -> N {
        self.a_mask
    }

    fn this(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::U2(self)
    }

    fn dgr(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::U2(Self {
            matrix: inverse_unitary_m2(&self.matrix),
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
        re: FRAC_1_SQRT_2,
        im: 0.0,
    };

    let op = Op::new(0b01, 0b10, [I, O, O, O, O, I, O, O, O, O, I, O, O, O, O, I]);
    assert!(op.is_valid());

    let op: SingleOp = op.into();
    assert_eq!(op.name(), format!("U3[[{I:?}, {O:?}, {O:?}, {O:?}], [{O:?}, {I:?}, {O:?}, {O:?}], [{O:?}, {O:?}, {I:?}, {O:?}], [{O:?}, {O:?}, {O:?}, {I:?}]]"));
    assert_eq!(
        op.matrix(2),
        [[I, O, O, O], [O, I, O, O], [O, O, I, O], [O, O, O, I],]
    );

    let op = Op::new(0b01, 0b10, [I, O, O, O, I, I, O, O, O, O, I, O, O, O, O, I]);
    assert!(!op.is_valid());

    let op = Op::new(0b11, 0b10, [I, O, O, O, O, I, O, O, O, O, I, O, O, O, O, I]);
    assert!(!op.is_valid());

    let op = Op::new(
        0b01,
        0b10,
        [
            SQRT_1_2, SQRT_1_2, O, O, SQRT_1_2, -SQRT_1_2, O, O, O, O, -SQRT_1_2, -SQRT_1_2, O, O,
            -SQRT_1_2, SQRT_1_2,
        ],
    );
    assert!(op.is_valid());

    let op: SingleOp = op.into();
    assert_eq!(op.name(), format!("U3[[{SQRT_1_2:?}, {SQRT_1_2:?}, {O:?}, {O:?}], [{SQRT_1_2:?}, {:?}, {O:?}, {O:?}], [{O:?}, {O:?}, {:?}, {:?}], [{O:?}, {O:?}, {:?}, {SQRT_1_2:?}]]", -SQRT_1_2, -SQRT_1_2, -SQRT_1_2, -SQRT_1_2));
    assert_eq!(
        op.matrix(2),
        [
            [SQRT_1_2, SQRT_1_2, O, O],
            [SQRT_1_2, -SQRT_1_2, O, O],
            [O, O, -SQRT_1_2, -SQRT_1_2],
            [O, O, -SQRT_1_2, SQRT_1_2],
        ]
    );
}
