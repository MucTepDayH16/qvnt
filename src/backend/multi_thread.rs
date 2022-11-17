#![allow(clippy::uninit_vec)]

use std::{fmt, num::NonZeroUsize, rc::Rc};

use rayon::{
    prelude::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
        IntoParallelRefMutIterator, ParallelIterator,
    },
    ThreadPool, ThreadPoolBuilder,
};

use super::{Backend, BackendBuilder, BackendResult};
use crate::{
    math::{approx_cmp::approx_eq_real, consts::*, types::*},
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

#[derive(Clone, Copy, Default, Debug)]
pub struct MultiThreadBuilder {
    pub num_threads: Option<NonZeroUsize>,
}

impl MultiThreadBuilder {
    pub fn with(num_threads: usize) -> Self {
        Self {
            num_threads: NonZeroUsize::new(num_threads),
        }
    }
}

impl BackendBuilder for MultiThreadBuilder {
    type Backend = MultiThread;

    fn build(self, q_num: N) -> BackendResult<Self::Backend> {
        let alloc_size = 1 << q_num;

        // It is safe to use uninitialized buffer
        // since it will be used only with `write` access
        let psi_buffer = uninit_vec(alloc_size);

        let num_threads = self
            .num_threads
            .map(NonZeroUsize::get)
            .unwrap_or_else(rayon::current_num_threads);

        let thread_pool = ThreadPoolBuilder::default()
            .num_threads(num_threads)
            .thread_name(|th_idx| format!("qvnt worker #{:?}", th_idx))
            .build()
            .map_err(|e| e.to_string())?;

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
    fn reset_state(&mut self, state: Mask) -> BackendResult {
        let MultiThread {
            thread_pool: _,
            psi_main,
            psi_buffer: _,
        } = self;
        psi_main.fill(C_ZERO);
        psi_main[state] = C_ONE;

        Ok(())
    }

    fn reset_state_and_size(&mut self, q_num: N, state: Mask) -> BackendResult {
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

    fn drain(&mut self) -> BackendResult<Vec<C>> {
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
        Ok(psi)
    }

    fn collect(&self) -> BackendResult<Vec<C>> {
        let MultiThread {
            thread_pool,
            psi_main,
            ..
        } = self;

        thread_pool.install(|| Ok(psi_main.par_iter().cloned().collect()))
    }

    fn collect_probabilities(&self) -> BackendResult<Vec<R>> {
        let MultiThread {
            thread_pool,
            psi_main,
            ..
        } = self;

        thread_pool.install(|| {
            let mut probs: Vec<_> = psi_main.par_iter().map(C::norm_sqr).collect();
            let inv_norm = 1. / probs.par_iter().sum::<R>();
            probs.par_iter_mut().for_each(|prob| *prob *= inv_norm);

            Ok(probs)
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

    fn apply_op(&mut self, op: &AtomicOpDispatch) -> BackendResult {
        let MultiThread {
            thread_pool,
            psi_main,
            psi_buffer,
        } = self;

        thread_pool.install(|| op.apply_for(&mut psi_buffer[..], &psi_main[..]));
        std::mem::swap(psi_main, psi_buffer);

        Ok(())
    }

    fn apply_op_controled(&mut self, op: &AtomicOpDispatch, ctrl: Mask) -> BackendResult {
        let MultiThread {
            thread_pool,
            psi_main,
            psi_buffer,
        } = self;

        thread_pool.install(|| op.apply_for_with_ctrl(&mut psi_buffer[..], &psi_main[..], ctrl));
        std::mem::swap(psi_main, psi_buffer);

        Ok(())
    }

    fn tensor_prod_assign(&mut self, other: Self) -> BackendResult {
        let MultiThread {
            thread_pool,
            psi_main,
            psi_buffer,
        } = self;
        let MultiThread {
            thread_pool: other_thread_pool,
            psi_main: other_psi,
            ..
        } = &other;

        if thread_pool.current_num_threads() < other_thread_pool.current_num_threads() {
            *thread_pool = other_thread_pool.clone();
        }

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

    fn collapse_by_mask(&mut self, collapse_state: Mask, mask: Mask) -> BackendResult {
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

        if approx_eq_real(abs, 0.0) {
            self.reset_state(0)?;
        } else if !approx_eq_real(abs, 1.0) {
            thread_pool.install(|| {
                psi_main.par_iter_mut().for_each(|psi| {
                    *psi /= abs;
                })
            });
        }

        Ok(())
    }
}

#[enum_dispatch::enum_dispatch(AtomicOpDispatch)]
pub(crate) trait MultiThreadOp {
    fn apply_for(&self, psi_out: &mut [C], psi_in: &[C]);

    fn apply_for_with_ctrl(&self, psi_out: &mut [C], psi_in: &[C], ctrl: Mask);
}

impl<NOp: NativeCpuOp> MultiThreadOp for NOp {
    fn apply_for(&self, psi_out: &mut [C], psi_in: &[C]) {
        psi_out
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, psi_out)| {
                *psi_out = self.native_cpu_op(psi_in, idx);
            })
    }

    fn apply_for_with_ctrl(&self, psi_out: &mut [C], psi_in: &[C], ctrl: Mask) {
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
