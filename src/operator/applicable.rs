use std::borrow::Borrow;
use crate::math::{C, R, N};

pub trait Applicable: Sized {
    fn apply(&self, psi: Vec<C>) -> Vec<C>;

    fn act_on(&self) -> N;

    fn dgr(self) -> Self;

    fn c(self, c_mask: N) -> Option<Self>;

    fn matrix(&self, size: N) -> Vec<Vec<C>> {
        const O: C = C{ re: 0.0, im: 0.0 };
        const I: C = C{ re: 1.0, im: 0.0 };

        let size = 1 << size;

        let mut matrix = vec![];
        matrix.reserve(size);

        for idx in 0..size {
            let mut psi = vec![];
            psi.resize(size, O);
            psi[idx] = I;

            matrix.push(self.apply(psi));
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