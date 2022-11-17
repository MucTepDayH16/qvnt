use super::*;

pub(crate) fn test_build(q_num: usize) -> BackendResult<TestBackend> {
    let b0 = SingleThreadBuilder::default().build(q_num)?;
    #[cfg(feature = "multi-thread")]
    let b1 = MultiThreadBuilder::default().build(q_num)?;

    Ok(TestBackend {
        b0,
        #[cfg(feature = "multi-thread")]
        b1,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct TestBackend {
    b0: SingleThread,
    #[cfg(feature = "multi-thread")]
    b1: MultiThread,
}

impl Backend for TestBackend {
    fn reset_state(&mut self, state: Mask) -> BackendResult {
        self.b0.reset_state(state)?;
        #[cfg(feature = "multi-thread")]
        self.b1.reset_state(state)?;

        Ok(())
    }

    fn reset_state_and_size(&mut self, q_num: N, state: Mask) -> BackendResult {
        self.b0.reset_state_and_size(q_num, state)?;
        #[cfg(feature = "multi-thread")]
        self.b1.reset_state_and_size(q_num, state)?;

        Ok(())
    }

    fn drain(&mut self) -> BackendResult<Vec<C>> {
        let vec0 = self.b0.drain()?;
        #[cfg(feature = "multi-thread")]
        {
            let vec1 = self.b1.drain()?;
            assert_eq!(vec0, vec1);
        }

        Ok(vec0)
    }

    fn collect(&self) -> BackendResult<Vec<C>> {
        let vec0 = self.b0.collect()?;
        #[cfg(feature = "multi-thread")]
        {
            let vec1 = self.b1.collect()?;
            assert_eq!(vec0, vec1);
        }

        Ok(vec0)
    }

    fn collect_probabilities(&self) -> BackendResult<Vec<R>> {
        let vec0 = self.b0.collect_probabilities()?;
        #[cfg(feature = "multi-thread")]
        {
            let vec1 = self.b1.collect_probabilities()?;
            assert_eq!(vec0, vec1);
        }

        Ok(vec0)
    }

    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.b0.fmt(fmt)
    }

    fn apply_op(&mut self, op: &AtomicOpDispatch) -> BackendResult {
        self.b0.apply_op(op)?;
        #[cfg(feature = "multi-thread")]
        self.b1.apply_op(op)?;

        Ok(())
    }

    fn apply_op_controled(&mut self, op: &AtomicOpDispatch, ctrl: Mask) -> BackendResult {
        self.b0.apply_op_controled(op, ctrl)?;
        #[cfg(feature = "multi-thread")]
        self.b1.apply_op_controled(op, ctrl)?;

        Ok(())
    }

    fn tensor_prod_assign(&mut self, other: Self) -> BackendResult {
        self.b0.tensor_prod_assign(other.b0)?;
        #[cfg(feature = "multi-thread")]
        self.b1.tensor_prod_assign(other.b1)?;

        Ok(())
    }

    fn collapse_by_mask(&mut self, collapse_state: Mask, mask: Mask) -> BackendResult {
        self.b0.collapse_by_mask(collapse_state, mask)?;
        #[cfg(feature = "multi-thread")]
        self.b1.collapse_by_mask(collapse_state, mask)?;

        Ok(())
    }
}

mod tests {
    use super::*;
    use crate::math::consts::*;

    #[test]
    fn init() {
        let mut backend = test_build(4).unwrap();
        backend.reset_state(0).unwrap();
        assert_eq!(
            backend.collect().unwrap(),
            [&[C_ONE; 1][..], &[C_ZERO; 15]].concat()
        );
    }
}
