use crate::math::{C, R, N};

type Ptr<T> = std::sync::Arc<T>;

pub (crate) trait AtomicOp: Send + Sync + std::panic::UnwindSafe {
    fn atomic_op(&self, psi: &[C], idx: N) -> C;

    fn name(&self) -> String;

    fn is_valid(&self) -> bool { true }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp>;
}

macro_rules! simple_op_impl {
    ($mask:ident) => {
        impl Op {
            #[inline(always)]
            pub fn new($mask: N) -> Self {
                Self{ $mask }
            }
        }
    }
}
macro_rules! rotate_op_impl {
    ($mask:ident) => {
        impl Op {
            #[inline(always)]
            pub fn new($mask: N, mut phase: R) -> Self {
                phase *= 0.5;
                let phase = C::new(phase.cos(), phase.sin());
                Self{ $mask, phase }
            }
        }
    }
}

pub (crate) mod id;

pub (crate) mod x;
pub (crate) mod rx;
pub (crate) mod rxx;

pub (crate) mod y;
pub (crate) mod ry;
pub (crate) mod ryy;

pub (crate) mod z;
pub (crate) mod s;
pub (crate) mod t;
pub (crate) mod rz;
pub (crate) mod rzz;

pub (crate) mod phi;

pub (crate) mod h1;
pub (crate) mod h2;

pub (crate) mod swap;
//pub (crate) mod iswap;
//pub (crate) mod sqrt_swap;
//pub (crate) mod sqrt_iswap;