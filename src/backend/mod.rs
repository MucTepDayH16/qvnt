use std::{convert::Infallible, fmt, str::FromStr};

use crate::{
    math::{Mask, C, N, R},
    operator::atomic::AtomicOpDispatch,
};

pub mod single_thread;

#[derive(Clone, Debug)]
pub enum BackendError {
    Custom(String),
}

impl FromStr for BackendError {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Custom(s.to_string()))
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

    fn tensor_prod_assign(&mut self, other_psi: Vec<C>) -> Result<(), BackendError>;

    fn collapse_by_mask(&mut self, collapse_state: Mask, mask: Mask) -> Result<(), BackendError>;
}

impl<Ref: AsRef<dyn Backend> + AsMut<dyn Backend>> Backend for Ref {
    fn reset_state(&mut self, state: Mask) -> Result<(), BackendError> {
        self.as_mut().reset_state(state)
    }

    fn reset_state_and_size(&mut self, q_num: N, state: Mask) -> Result<(), BackendError> {
        self.as_mut().reset_state_and_size(q_num, state)
    }

    fn drain(&mut self) -> Vec<C> {
        self.as_mut().drain()
    }

    fn collect(&self) -> Vec<C> {
        self.as_ref().collect()
    }

    fn collect_probabilities(&self) -> Vec<R> {
        self.as_ref().collect_probabilities()
    }

    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(fmt)
    }

    fn apply_op_controled(
        &mut self,
        op: &AtomicOpDispatch,
        ctrl: Mask,
    ) -> Result<(), BackendError> {
        self.as_mut().apply_op_controled(op, ctrl)
    }

    fn apply_op(&mut self, op: &AtomicOpDispatch) -> Result<(), BackendError> {
        self.as_mut().apply_op(op)
    }

    fn tensor_prod_assign(&mut self, other_psi: Vec<C>) -> Result<(), BackendError> {
        self.as_mut().tensor_prod_assign(other_psi)
    }

    fn collapse_by_mask(&mut self, collapse_state: Mask, mask: Mask) -> Result<(), BackendError> {
        self.as_mut().collapse_by_mask(collapse_state, mask)
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::{single_thread::SingleThread, Backend, BackendBuilder};

    #[test]
    fn assert_object_save() {
        fn _assert_object_safe() -> Box<dyn Backend + 'static> {
            Box::new(SingleThread {
                psi_main: vec![],
                psi_buffer: vec![],
            })
        }

        fn _assert_has_trait(_: impl Backend) {}

        _assert_has_trait(_assert_object_safe());

        (|_| Ok(_assert_object_safe())).build(0).unwrap();
    }
}
