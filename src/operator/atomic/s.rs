use super::*;

pub (crate) struct Op {
    a_mask: N,
    dagger: bool,
}

impl Op {
    #[inline(always)]
    pub fn new(a_mask: N) -> Self {
        Self{ a_mask, dagger: false }
    }
}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut count = (idx & self.a_mask).count_ones() as usize;
        if self.dagger { count = (!count).wrapping_add(1); }
        crate::math::rotate(psi[idx], count)
    }

    fn name(&self) -> String {
        format!("S{}", self.a_mask)
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        Ptr::new(Self{ dagger: !self.dagger, ..*self })
    }
}

#[cfg(test)] #[test]
fn tests() {
    use crate::operator::single::*;

    const O: C = C{ re: 0.0, im: 0.0 };
    const I: C = C{ re: 1.0, im: 0.0 };
    const i: C = C{ re: 0.0, im: 1.0 };

    let op = SingleOp::from_atomic(Op::new(0b1)).unwrap().dgr();
    assert_eq!(op.name(), "S1");
    assert_eq!(op.matrix(1),
               [   [I, O],
                   [O, -i]   ]);
}