use std::{boxed::Box as Ptr};
use std::ops::Deref;
use super::single::SingleOp;
use crate::math::{C, R, N};

pub (crate) trait AtomicOp: Sync + Send {
    fn atomic_op(&self, psi: &[C], idx: N) -> C;

    fn name(&self) -> String;

    fn is_valid(&self) -> bool {
        true
    }

    fn dgr(&self) -> Box<dyn AtomicOp>;

    fn cloned(&self) -> Box<dyn AtomicOp>;
}

impl Clone for Box<dyn AtomicOp> {
    fn clone(&self) -> Self {
        self.deref().cloned()
    }
}

macro_rules! clone_impl {
    () => {
        fn cloned(&self) -> Box<dyn AtomicOp> {
            Box::new(self.clone())
        }
    }
}
macro_rules! op_impl {
    (s $mask:ident) => {
        #[derive(Clone, Copy)]
        pub(crate) struct Op {
            $mask: N,
        }

        impl Op {
            #[inline(always)]
            pub fn new($mask: N) -> Self {
                Self { $mask }
            }
        }

        into_single_op_impl! { $mask }
    };
    (d $mask:ident) => {
        #[derive(Clone, Copy)]
        pub(crate) struct Op {
            $mask: N,
            dagger: bool,
        }

        impl Op {
            #[inline(always)]
            pub fn new($mask: N) -> Self {
                Self {
                    $mask,
                    dagger: false,
                }
            }
        }

        into_single_op_impl! { $mask }
    };
    (r $mask:ident) => {
        #[derive(Clone, Copy)]
        pub(crate) struct Op {
            $mask: N,
            phase: C,
        }

        impl Op {
            #[inline(always)]
            pub fn new($mask: N, mut phase: R) -> Self {
                phase *= 0.5;
                let phase = C::new(phase.cos(), phase.sin());
                Self { $mask, phase }
            }
        }

        into_single_op_impl! { $mask }
    };
}
macro_rules! into_single_op_impl {
    ($mask:ident) => {
        impl Into<SingleOp> for Op {
            fn into(self) -> SingleOp {
                SingleOp {
                    act: self.$mask,
                    ctrl: 0,
                    func: Ptr::new(self),
                }
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