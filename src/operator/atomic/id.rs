pub use super::*;

#[derive(Clone, Copy)]
pub (crate) struct Op ();

impl Into<SingleOp> for Op {
    fn into(self) -> SingleOp {
        SingleOp { act: 0, ctrl: 0, func: Ptr::new(self) }
    }
}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        psi[idx]
    }

    fn name(&self) -> String {
        "Id".to_string()
    }

    fn dgr(&self) -> Box<dyn AtomicOp> {
        Box::new(*self)
    }

    clone_impl!{}
}

#[cfg(test)] #[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C{ re: 0.0, im: 0.0 };
    const I: C = C{ re: 1.0, im: 0.0 };

    let op: SingleOp = Op().into();
    assert_eq!(op.name(), "Id");
    assert_eq!(op.matrix(1),
               [   [I, O],
                   [O, I]   ]);
}