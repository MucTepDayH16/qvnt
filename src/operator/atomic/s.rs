use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op {
    a_mask: Mask,
    dagger: bool,
}

impl Op {
    pub fn new(a_mask: Mask) -> Self {
        Self {
            a_mask,
            dagger: false,
        }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    #[inline(always)]
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        let mut count = (idx & self.a_mask).count_ones() as usize;
        if self.dagger {
            count = (!count).wrapping_add(1);
        }
        crate::math::rotate(psi[idx], count)
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("S{}", self.a_mask)
    }

    fn acts_on(&self) -> Mask {
        self.a_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::S(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::S(Self {
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

    let op: SingleOp = Op::new(0b1).into();
    let op = op.dgr();
    assert_eq!(op.name(), "S1");
    assert_eq!(op.matrix(1), [[I, O], [O, -i]]);
}
