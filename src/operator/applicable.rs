use crate::math::{C, R, N};

pub trait Applicable {
    fn apply(&self, _: Vec<C>) -> Vec<C>;

    //#[cfg(any(test,doc))]
    fn matrix(&self, size: N) -> Vec<Vec<C>> {
        const Z: C = C{ re: 0.0, im: 0.0 };
        const O: C = C{ re: 1.0, im: 0.0 };

        let size = 1 << size;

        let mut matrix = vec![];
        matrix.reserve(size);

        for idx in 0..size {
            let mut psi = vec![];
            psi.resize(size, Z);
            psi[idx] = O;

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