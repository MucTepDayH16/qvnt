use super::*;

pub (crate) struct Op {
    a_mask: N,
    phase: C,
}

rotate_op_impl!{a_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let psi = (psi[idx], psi[idx ^ self.a_mask]);
        C { re: psi.0.re * self.phase.re + psi.1.im * self.phase.im,
            im: psi.0.im * self.phase.re - psi.1.re * self.phase.im }
    }

    fn name(&self) -> String {
        format!("RX{}[{}]", self.a_mask, 2.0 * self.phase.arg())
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        Ptr::new(Self{ phase: -self.phase, ..*self })
    }
}

#[cfg(test)] #[test]
fn tests() {
    use crate::operator::single::*;

    const ANGLE: R = 1.23456;

    let cos = C{ re: (0.5 * ANGLE).cos(), im: 0.0 };
    let i_sin = C{ re: 0.0, im: (0.5 * ANGLE).sin() };

    let op = SingleOp::from_atomic(Op::new(0b1, ANGLE)).unwrap();
    assert_eq!(op.name(), "RX1[1.23456]");
    assert_eq!(op.matrix(1),
               [   [cos, -i_sin],
                   [-i_sin, cos]   ]);
}