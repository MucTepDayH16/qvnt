use std::{convert::Infallible, fmt, str::FromStr};

use crate::{math::types::*, operator::atomic::AtomicOpDispatch};

#[cfg(feature = "multi-thread")]
pub mod multi_thread;
pub mod single_thread;

#[derive(Clone, Debug)]
pub enum BackendError {
    Custom(String),
}

impl FromStr for BackendError {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.to_string().into())
    }
}

impl From<String> for BackendError {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

pub type DefaultBuilder = single_thread::SingleThreadBuilder;

pub trait BackendBuilder: Sized {
    type Backend: Backend;

    fn build(self, q_num: N) -> Result<Self::Backend, BackendError>;
}

impl<B: Backend, Func: FnOnce(N) -> Result<B, BackendError>> BackendBuilder for Func {
    type Backend = B;

    fn build(self, q_num: N) -> Result<Self::Backend, BackendError> {
        self(q_num)
    }
}

pub trait Backend {
    fn reset_state(&mut self, state: Mask) -> Result<(), BackendError>;

    fn reset_state_and_size(&mut self, q_num: N, state: Mask) -> Result<(), BackendError>;

    fn drain(&mut self) -> Vec<C>;

    fn collect(&self) -> Vec<C>;

    fn collect_probabilities(&self) -> Vec<R> {
        let mut probs: Vec<_> = self
            .collect()
            .into_iter()
            .map(|psi| psi.norm_sqr())
            .collect();
        let inv_norm = 1. / probs.iter().sum::<R>();
        probs.iter_mut().for_each(|psi| *psi *= inv_norm);

        probs
    }

    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result;

    fn apply_op_controled(&mut self, op: &AtomicOpDispatch, ctrl: Mask)
        -> Result<(), BackendError>;

    fn apply_op(&mut self, op: &AtomicOpDispatch) -> Result<(), BackendError> {
        Self::apply_op_controled(self, op, 0)
    }

    fn tensor_prod_assign(&mut self, other: Self) -> Result<(), BackendError>;

    fn collapse_by_mask(&mut self, collapse_state: Mask, mask: Mask) -> Result<(), BackendError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{multi_thread::MultiThreadBuilder, single_thread::SingleThreadBuilder};

    #[test]
    fn fn_builder() {
        fn custom_build<B: BackendBuilder>(b: B) -> B::Backend {
            b.build(1).unwrap()
        }

        let _ = custom_build(SingleThreadBuilder);
        let _ = custom_build(MultiThreadBuilder::default());
        let _ = custom_build(|q_num| SingleThreadBuilder.build(q_num));
    }
}
