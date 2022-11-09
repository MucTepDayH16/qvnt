#![allow(clippy::uninit_vec)]

use std::fmt;

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

#[derive(Clone, Copy, Default, Debug)]
pub struct SingleThreadBuilder;

impl BackendBuilder for SingleThreadBuilder {
    type Backend = SingleThread;

    fn build(self, q_num: N) -> Result<Self::Backend, BackendError> {
        let alloc_size = 1 << q_num;

        // It is safe to use uninitialized buffer
        // since it will be used only with `write` access
        let psi_buffer = uninit_vec(alloc_size);

        Ok(SingleThread {
            psi_main: vec![C_ZERO; alloc_size],
            psi_buffer,
        })
    }
}

#[derive(Default, Debug)]
pub struct SingleThread {
    pub(crate) psi_main: Vec<C>,
    pub(crate) psi_buffer: Vec<C>,
}

impl Clone for SingleThread {
    fn clone(&self) -> Self {
        let size = self.psi_main.len();
        let psi_buffer = uninit_vec(size);

        Self {
            psi_main: self.psi_main.clone(),
            psi_buffer,
        }
    }
}

impl Drop for SingleThread {
    fn drop(&mut self) {}
}

impl Backend for SingleThread {
    fn reset_state(&mut self, state: Mask) -> Result<(), BackendError> {
        self.psi_main.fill(C_ZERO);
        self.psi_main[state] = C_ONE;

        Ok(())
    }

    fn reset_state_and_size(&mut self, q_num: N, state: Mask) -> Result<(), BackendError> {
        let new_size = 1usize << q_num;

        let old_size = self.psi_main.len();

        if new_size == old_size {
            return Ok(());
        }

        self.psi_main.fill_with(|| C_ZERO);
        self.psi_main.resize_with(new_size, || C_ZERO);
        self.psi_main[state] = C_ONE;

        if new_size > old_size {
            let additional = new_size - old_size;
            self.psi_buffer.reserve_exact(additional);
            unsafe {
                self.psi_buffer.set_len(new_size);
            }
        } else if new_size < old_size {
            unsafe {
                self.psi_buffer.set_len(new_size);
            }
            self.psi_buffer.shrink_to_fit();
        }

        Ok(())
    }

    fn drain(&mut self) -> Vec<C> {
        let size = self.psi_main.len();
        let mut psi = std::mem::take(&mut self.psi_main);
        unsafe {
            psi.set_len(size);
        }
        self.psi_buffer.clear();
        psi
    }

    fn collect(&self) -> Vec<C> {
        self.psi_main.clone()
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.psi_main.len() <= MAX_LEN_TO_DISPLAY {
            self.psi_main
                .iter()
                .enumerate()
                .fold(&mut f.debug_struct("QReg"), |f, (idx, psi)| {
                    f.field(&format!("{}", idx), psi)
                })
                .finish()
        } else {
            self.psi_main[..MAX_LEN_TO_DISPLAY]
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
        let SingleThread {
            psi_main,
            psi_buffer,
            ..
        } = self;

        op.apply_for_each(&mut psi_buffer[..], &psi_main[..], ctrl);
        std::mem::swap(psi_main, psi_buffer);

        Ok(())
    }

    fn tensor_prod_assign(&mut self, other_psi: Vec<C>) -> Result<(), BackendError> {
        let self_size = self.psi_main.len();
        let new_len = self_size.checked_mul(other_psi.len()).unwrap();
        let self_mask = self_size.saturating_sub(1);

        self.psi_main = (0..new_len)
            .map(|idx| {
                let self_idx = idx & self_mask;
                let other_idx = idx >> self_size;

                unsafe {
                    self.psi_main.get_unchecked(self_idx) * other_psi.get_unchecked(other_idx)
                }
            })
            .collect();
        self.psi_buffer = uninit_vec(new_len);
        Ok(())
    }

    fn collapse_by_mask(&mut self, collapse_state: Mask, mask: Mask) -> Result<(), BackendError> {
        let abs = self
            .psi_main
            .iter_mut()
            .enumerate()
            .fold(0.0, |abs_sqr, (idx, psi)| {
                if (idx ^ collapse_state) & mask != 0 {
                    *psi = C_ZERO;
                    abs_sqr
                } else {
                    abs_sqr + psi.norm_sqr()
                }
            })
            .sqrt();

        if approx_cmp(abs, 0.0) {
            self.reset_state(0)?;
        } else if !approx_cmp(abs, 1.0) {
            self.psi_main.iter_mut().for_each(|psi| {
                *psi /= abs;
            });
        }

        Ok(())
    }
}

#[::dispatch::enum_dispatch(AtomicOpDispatch)]
pub(crate) trait SingleThreadOp {
    fn apply_for_each(&self, psi_out: &mut [C], psi_in: &[C], ctrl: Mask);
}

impl<NOp: NativeCpuOp> SingleThreadOp for NOp {
    fn apply_for_each(&self, psi_out: &mut [C], psi_in: &[C], ctrl: Mask) {
        for (idx, psi_out) in psi_out.iter_mut().enumerate() {
            *psi_out = if !idx & ctrl == 0 {
                self.native_cpu_op(psi_in, idx)
            } else {
                unsafe { *psi_in.get_unchecked(idx) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        let mut backend = SingleThreadBuilder.build(4).unwrap();
        backend.reset_state(0).unwrap();
        assert_eq!(backend.psi_main, [&[C_ONE; 1][..], &[C_ZERO; 15]].concat());
    }
}
