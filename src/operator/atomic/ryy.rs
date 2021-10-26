use super::*;

op_impl!{r ab_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut phase = self.phase;
        let psi = (psi[idx], psi[idx ^ self.ab_mask]);
        if (idx & self.ab_mask).count_ones() & 1 == 0 { phase.im = -phase.im; }
        C { re: psi.0.re * phase.re + psi.1.im * phase.im,
            im: psi.0.im * phase.re - psi.1.re * phase.im }
    }

    fn name(&self) -> String {
        format!("RYY{}[{}]", self.ab_mask, 2.0 * self.phase.arg())
    }

    fn is_valid(&self) -> bool {
        self.ab_mask.count_ones() == 2
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        Ptr::new(Self{ phase: -self.phase, ..*self })
    }
}

#[cfg(test)] #[test]
fn tests() {
    use crate::operator::single::*;

    const ANGLE: R = 1.23456;

    const O: C = C{ re: 0.0, im: 0.0 };
    let cos = C{ re: (0.5 * ANGLE).cos(), im: 0.0 };
    let i_sin = C{ re: 0.0, im: (0.5 * ANGLE).sin() };

    let op: SingleOp = Op::new(0b11,ANGLE).into();
    assert_eq!(op.name(), "RYY3[1.23456]");
    assert_eq!(op.matrix(2),
               [   [cos,    O,      O,      i_sin   ],
                   [O,      cos,    -i_sin, O       ],
                   [O,      -i_sin, cos,    O       ],
                   [i_sin,  O,      O,      cos     ]   ]);
}