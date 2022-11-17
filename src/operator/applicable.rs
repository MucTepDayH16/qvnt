use crate::{
    backend::{Backend, BackendResult},
    math::types::*,
};

pub trait Applicable: Sized + Sync {
    fn apply(&self, backend: &mut impl Backend) -> BackendResult;

    fn act_on(&self) -> Mask;

    fn dgr(self) -> Self;

    fn c(self, c_mask: Mask) -> Option<Self>;

    #[cfg(test)]
    fn matrix(&self, q_num: N) -> Vec<Vec<C>> {
        let size = 1 << q_num;

        const O: C = C { re: 0.0, im: 0.0 };
        const I: C = C { re: 1.0, im: 0.0 };

        let mut matrix = vec![];
        matrix.reserve(q_num);

        let init_and_apply_op = |idx: Mask| -> BackendResult<Vec<C>> {
            let mut backend_data = crate::backend::test_backend::test_build(q_num)?;
            backend_data.reset_state(idx)?;
            self.apply(&mut backend_data)?;
            backend_data.collect()
        };

        for idx in 0..size {
            let psi = init_and_apply_op(idx).unwrap();
            matrix.push(psi);
        }

        for idx in 0..size {
            for jdx in 0..idx {
                let tmp = matrix[idx][jdx];
                matrix[idx][jdx] = matrix[jdx][idx];
                matrix[jdx][idx] = tmp;
            }
        }

        matrix
    }
}
