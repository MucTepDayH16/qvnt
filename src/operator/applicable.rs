use crate::{
    backend::{Backend, BackendError},
    math::types::*,
};

pub trait Applicable: Sized + Sync {
    fn apply(&self, backend: &mut impl Backend) -> Result<(), BackendError>;

    fn act_on(&self) -> Mask;

    fn dgr(self) -> Self;

    fn c(self, c_mask: Mask) -> Option<Self>;

    #[cfg(test)]
    fn matrix(&self, q_num: N) -> Vec<Vec<C>> {
        use crate::backend::{single_thread::SingleThreadBuilder as B, BackendBuilder};

        let size = 1 << q_num;

        const O: C = C { re: 0.0, im: 0.0 };
        const I: C = C { re: 1.0, im: 0.0 };

        let mut matrix = vec![];
        matrix.reserve(q_num);

        for idx in 0..size {
            let mut backend_data = B.build(q_num).unwrap();
            backend_data.reset_state(idx).unwrap();
            self.apply(&mut backend_data).unwrap();

            matrix.push(backend_data.collect());
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
