pub(crate) use super::Applicable;
use crate::{math::types::*, operator::atomic::*};

macro_rules! single_op_checked {
    ($op:expr) => {
        match $op {
            op if op.is_valid() => Some(op.into()),
            _ => None,
        }
    };
}

pub mod pauli;
pub mod rotate;
pub mod swap;

/// Single quantum operation.
///
/// This structure represents the unit of computation for quantum simulator.
/// Since [`SingleOp`] does not have public constructor, it only can be acquired by indexing [`MultiOp`](super::MultiOp):
///
/// ```rust
/// # use qvnt::prelude::*;
/// let multi_op: MultiOp = op::x(0b1);
/// // Since SingleOp does not implement "Copy" trait,
/// // it only can be referenced or cloned
/// let single_op: SingleOp = op::x(0b1)[0].clone();
/// ```
///
/// As [`MultiOp`](super::MultiOp), it could be applied to [`QReg`](crate::prelude::QReg):
///
/// ```rust
/// # use qvnt::prelude::*;
/// let mut reg = QReg::new(1);
///
/// reg.apply(&op::x(0b1)[0]);
/// ```
///
/// This is similar to reg.apply(&op::x(0b1)).
/// Using index notation you could deconstruct complex gates (e.g. [`Quantum Fourier Transform`](super::qft()))
/// into simple ones and apply them *insequentially*.
#[derive(Clone, PartialEq)]
pub struct SingleOp {
    act: N,
    ctrl: N,
    func: dispatch::AtomicOpDispatch,
}

impl SingleOp {
    /// Return 'name' of quantum gate.
    /// It is formatted as ```(C{control_mask}_)?{gate_name}{apply_mask}```, where:
    /// * gate_name     - Similar to gate's name in [OpenQASM standard](https://en.wikipedia.org/wiki/OpenQASM);
    /// * control_mask  - [mask] for controlled qubits.
    /// If equals 0, ```C{control_mask}_``` will not be displayed;
    /// * apply_mask    - [mask] for qubits affected by the given gate.
    ///
    /// ```rust
    /// # use qvnt::prelude::*;
    /// let single_op = &op::x(123)[0];
    /// println!("{}", single_op.name());
    /// // which is similar to this:
    /// // println!("{:?}", single_op);
    ///
    /// let controlled_op = single_op.clone().c(4).unwrap();
    /// println!("{:?}", controlled_op);
    /// ```
    ///
    /// Output will be:
    ///
    /// ```ignore
    /// X123
    /// C4_X123
    /// ```
    pub fn name(&self) -> String {
        if self.ctrl != 0 {
            format!("C{}_", self.ctrl) + &self.func.name()
        } else {
            self.func.name()
        }
    }
}

impl Applicable for SingleOp {
    fn apply(&self, psi_i: &[C], psi_o: &mut Vec<C>) {
        let ctrl = self.ctrl;
        self.func.for_each(psi_i, &mut psi_o[..], ctrl);
    }

    #[cfg(feature = "multi-thread")]
    fn apply_sync(&self, psi_i: &[C], psi_o: &mut Vec<C>) {
        let ctrl = self.ctrl;
        self.func.for_each_par(psi_i, &mut psi_o[..], ctrl);
    }

    #[inline]
    fn act_on(&self) -> N {
        self.act | self.ctrl
    }

    #[inline]
    fn dgr(self) -> Self {
        Self {
            func: self.func.dgr(),
            ..self
        }
    }

    #[inline(always)]
    fn c(self, c: N) -> Option<Self> {
        if self.act_on() & c != 0 {
            None
        } else {
            Some(Self {
                ctrl: self.ctrl | c,
                ..self
            })
        }
    }
}

impl<Op: AtomicOp> From<Op> for SingleOp {
    fn from(op: Op) -> Self {
        Self {
            act: op.acts_on(),
            ctrl: 0,
            func: op.this(),
        }
    }
}

impl std::fmt::Debug for SingleOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        let single_op = pauli::x(123);
        assert_eq!(single_op.name(), format!("X123"));
        assert_eq!(format!("{:?}", single_op), format!("X123"));

        let single_op = single_op.c(4).unwrap();
        assert_eq!(single_op.name(), format!("C4_X123"));
        assert_eq!(format!("{:?}", single_op), format!("C4_X123"));
    }

    #[test]
    fn unwrap_op() {
        assert!(rotate::ryy(0b001, 1.35).is_none());
        assert!(rotate::ryy(0b101, 1.35).unwrap().c(0b001).is_none());
        let _ = rotate::ryy(0b101, 1.35).unwrap().c(0b010).unwrap();

        assert!(swap::swap(0b001).is_none());
        assert!(swap::swap(0b101).unwrap().c(0b100).is_none());
        let _ = swap::swap(0b101).unwrap().c(0b010).unwrap();
    }
}
