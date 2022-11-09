#![allow(clippy::uninit_vec)]

use std::{fmt, rc::Rc};

use rayon::{
    prelude::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
        IntoParallelRefMutIterator, ParallelIterator,
    },
    ThreadPool, ThreadPoolBuilder,
};

use super::{Backend, BackendBuilder, BackendError};
use crate::{
    math::{approx_cmp::approx_cmp, types::*, C_ONE, C_ZERO},
    operator::atomic::{AtomicOpDispatch, NativeCpuOp},
};

const MAX_LEN_TO_DISPLAY: N = 8;

fn uninit_vec<T>(size: N) -> Vec<T> {
    let mut buffer = Vec::with_capacity(size);
    unsafe {
        buffer.set_len(size);
    }
    buffer
}

#[derive(Default, Clone, Copy)]
pub struct MultiThreadBuilder {
    pub num_threads: Option<usize>,
}

impl BackendBuilder for MultiThreadBuilder {
    type Backend = MultiThread;

    fn build(self, q_num: N) -> Result<Self::Backend, BackendError> {
        let alloc_size = 1 << q_num;

        // It is safe to use uninitialized buffer
        // since it will be used only with `write` access
        let psi_buffer = uninit_vec(alloc_size);

        let mut thread_pool_builder = ThreadPoolBuilder::default();
        if let Some(num_threads) = self.num_threads {
            thread_pool_builder = thread_pool_builder.num_threads(num_threads);
        }

        let thread_pool = thread_pool_builder.build().map_err(|e| e.to_string())?;

        Ok(MultiThread {
            thread_pool: Rc::new(thread_pool),
            psi_main: vec![C_ZERO; alloc_size],
            psi_buffer,
        })
    }
}

#[derive(Debug)]
pub struct MultiThread {
    pub(crate) thread_pool: Rc<ThreadPool>,
    pub(crate) psi_main: Vec<C>,
    pub(crate) psi_buffer: Vec<C>,
}

impl Clone for MultiThread {
    fn clone(&self) -> Self {
        let size = self.psi_main.len();
        let psi_buffer = uninit_vec(size);

        Self {
            thread_pool: self.thread_pool.clone(),
            psi_main: self.psi_main.clone(),
            psi_buffer,
        }
    }
}

impl Drop for MultiThread {
    fn drop(&mut self) {}
}

impl Backend for MultiThread {
    fn reset_state(&mut self, state: Mask) -> Result<(), BackendError> {
        let MultiThread {
            thread_pool: _,
            psi_main,
            psi_buffer: _,
        } = self;
        psi_main.fill(C_ZERO);
        psi_main[state] = C_ONE;

        Ok(())
    }

    fn reset_state_and_size(&mut self, q_num: N, state: Mask) -> Result<(), BackendError> {
        let MultiThread {
            thread_pool: _,
            psi_main,
            psi_buffer,
        } = self;

        let new_size = 1usize << q_num;

        let old_size = psi_main.len();

        if new_size == old_size {
            return Ok(());
        }

        psi_main.fill_with(|| C_ZERO);
        psi_main.resize_with(new_size, || C_ZERO);
        psi_main[state] = C_ONE;

        if new_size > old_size {
            let additional = new_size - old_size;
            psi_buffer.reserve_exact(additional);
            unsafe {
                psi_buffer.set_len(new_size);
            }
        } else if new_size < old_size {
            unsafe {
                psi_buffer.set_len(new_size);
            }
            psi_buffer.shrink_to_fit();
        }

        Ok(())
    }

    fn drain(&mut self) -> Vec<C> {
        let MultiThread {
            thread_pool: _,
            psi_main,
            psi_buffer,
        } = self;

        let size = psi_main.len();
        let mut psi = std::mem::take(psi_main);
        unsafe {
            psi.set_len(size);
        }
        psi_buffer.clear();
        psi
    }

    fn collect(&self) -> Vec<C> {
        let MultiThread {
            thread_pool,
            psi_main,
            ..
        } = self;

        thread_pool.install(|| psi_main.par_iter().cloned().collect())
    }

    fn collect_probabilities(&self) -> Vec<R> {
        let MultiThread {
            thread_pool,
            psi_main,
            ..
        } = self;

        thread_pool.install(|| {
            let mut probs: Vec<_> = psi_main.par_iter().map(C::norm_sqr).collect();
            let inv_norm = 1. / probs.par_iter().sum::<R>();
            probs.par_iter_mut().for_each(|prob| *prob *= inv_norm);

            probs
        })
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MultiThread { psi_main, .. } = self;

        if psi_main.len() <= MAX_LEN_TO_DISPLAY {
            psi_main
                .iter()
                .enumerate()
                .fold(&mut f.debug_struct("QReg"), |f, (idx, psi)| {
                    f.field(&format!("{}", idx), psi)
                })
                .finish()
        } else {
            psi_main[..MAX_LEN_TO_DISPLAY]
                .iter()
                .enumerate()
                .fold(&mut f.debug_struct("QReg"), |f, (idx, psi)| {
                    f.field(&format!("{}", idx), psi)
                })
                .finish_non_exhaustive()
        }
    }

    fn apply_op_controled(
        &mut self,
        op: &AtomicOpDispatch,
        ctrl: Mask,
    ) -> Result<(), BackendError> {
        let MultiThread {
            thread_pool,
            psi_main,
            psi_buffer,
        } = self;

        thread_pool.install(|| op.apply_for_each(&mut psi_buffer[..], &psi_main[..], ctrl));
        std::mem::swap(psi_main, psi_buffer);

        Ok(())
    }

    fn tensor_prod_assign(&mut self, other_psi: Vec<C>) -> Result<(), BackendError> {
        let MultiThread {
            thread_pool,
            psi_main,
            psi_buffer,
        } = self;

        let self_size = psi_main.len();
        let new_len = self_size.checked_mul(other_psi.len()).unwrap();
        let self_mask = self_size.saturating_sub(1);

        *psi_main = thread_pool.install(|| {
            (0..new_len)
                .into_par_iter()
                .map(|idx| {
                    let self_idx = idx & self_mask;
                    let other_idx = idx >> self_size;

                    unsafe { psi_main.get_unchecked(self_idx) * other_psi.get_unchecked(other_idx) }
                })
                .collect()
        });
        *psi_buffer = uninit_vec(new_len);

        Ok(())
    }

    fn collapse_by_mask(&mut self, collapse_state: Mask, mask: Mask) -> Result<(), BackendError> {
        let MultiThread {
            thread_pool,
            psi_main,
            ..
        } = self;

        let abs = thread_pool.install(|| {
            psi_main
                .par_iter_mut()
                .enumerate()
                .fold(
                    || 0.0,
                    |abs_sqr, (idx, psi)| {
                        if (idx ^ collapse_state) & mask != 0 {
                            *psi = C_ZERO;
                            abs_sqr
                        } else {
                            abs_sqr + psi.norm_sqr()
                        }
                    },
                )
                .sum::<R>()
                .sqrt()
        });

        if approx_cmp(abs, 0.0) {
            self.reset_state(0)?;
        } else if !approx_cmp(abs, 1.0) {
            thread_pool.install(|| {
                psi_main.par_iter_mut().for_each(|psi| {
                    *psi /= abs;
                })
            });
        }

        Ok(())
    }
}

#[::dispatch::enum_dispatch(AtomicOpDispatch)]
pub(crate) trait MultiThreadOp {
    fn apply_for_each(&self, psi_out: &mut [C], psi_in: &[C], ctrl: Mask);
}

impl<NOp: NativeCpuOp> MultiThreadOp for NOp {
    fn apply_for_each(&self, psi_out: &mut [C], psi_in: &[C], ctrl: Mask) {
        psi_out
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, psi_out)| {
                *psi_out = if !idx & ctrl == 0 {
                    self.native_cpu_op(psi_in, idx)
                } else {
                    unsafe { *psi_in.get_unchecked(idx) }
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        let mut backend = MultiThreadBuilder::default().build(4).unwrap();
        backend.reset_state(0).unwrap();
        assert_eq!(backend.psi_main, [&[C_ONE; 1][..], &[C_ZERO; 15]].concat());
    }
}