use super::single::SingleOp;
use crate::math::{C, R, N};

type Ptr<T> = std::sync::Arc<T>;

pub (crate) trait AtomicOp: Send + Sync {
    fn atomic_op(&self, psi: &[C], idx: N) -> C;

    fn name(&self) -> String;

    fn is_valid(&self) -> bool { true }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp>;
}

macro_rules! op_impl {
    (s $mask:ident) => {
        pub (crate) struct Op {
            $mask: N,
        }

        impl Op {
            #[inline(always)]
            pub fn new($mask: N) -> Self {
                Self{ $mask }
            }
        }

        impl Into<SingleOp> for Op {
            fn into(self) -> SingleOp {
                SingleOp { act: self.$mask, ctrl: 0, func: Ptr::new(self) }
            }
        }
    };
    (d $mask:ident) => {
        pub (crate) struct Op {
            $mask: N,
            dagger: bool
        }

        impl Op {
            #[inline(always)]
            pub fn new($mask: N) -> Self {
                Self{ $mask, dagger: false }
            }
        }

        impl Into<SingleOp> for Op {
            fn into(self) -> SingleOp {
                SingleOp { act: self.$mask, ctrl: 0, func: Ptr::new(self) }
            }
        }
    };
    (r $mask:ident) => {
        pub (crate) struct Op {
            $mask: N,
            phase: C,
        }

        impl Op {
            #[inline(always)]
            pub fn new($mask: N, mut phase: R) -> Self {
                phase *= 0.5;
                let phase = C::new(phase.cos(), phase.sin());
                Self{ $mask, phase }
            }
        }

        impl Into<SingleOp> for Op {
            fn into(self) -> SingleOp {
                SingleOp { act: self.$mask, ctrl: 0, func: Ptr::new(self) }
            }
        }
    }
}
macro_rules! into_single_op_impl {
    ($mask:ident) => {
        impl Into<SingleOp> for Op {
            fn into(self) -> SingleOp {
                SingleOp { act: self.$mask, ctrl: 0, func: Ptr::new(self) }
            }
        }
    };
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
pub (crate) mod i_swap;
pub (crate) mod sqrt_swap;
pub (crate) mod sqrt_i_swap;