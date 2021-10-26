use super::*;

op_impl!{r a_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut phase = self.phase;
        let psi = (psi[idx], psi[idx ^ self.a_mask]);
        if idx & self.a_mask == 0 { phase.im = -phase.im; }
        C { re: psi.0.re * phase.re + psi.1.re * phase.im,
            im: psi.0.im * phase.re + psi.1.im * phase.im }
    }

    fn name(&self) -> String {
        format!("RY{}[{}]", self.a_mask, 2.0 * self.phase.arg())
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
    let sin = C{ re: (0.5 * ANGLE).sin(), im: 0.0 };

    let op: SingleOp = Op::new(0b1, ANGLE).into();
    assert_eq!(op.name(), "RY1[1.23456]");
    assert_eq!(op.matrix(1),
               [   [cos, -sin],
                   [sin, cos]   ]);
}