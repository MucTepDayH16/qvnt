pub use super::*;

pub (crate) struct Op ();

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        psi[idx]
    }

    fn name(&self) -> String {
        "Id".to_string()
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        self
    }
}

#[cfg(test)] #[test]
fn tests() {
    use crate::operator::single::*;

    const O: C = C{ re: 0.0, im: 0.0 };
    const I: C = C{ re: 1.0, im: 0.0 };

    let op = SingleOp::from_atomic(id::Op()).unwrap();
    assert_eq!(op.name(), "Id");
    assert_eq!(op.matrix(1),
               [   [I, O],
                   [O, I]   ]);
}