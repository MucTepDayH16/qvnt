use super::int::*;
use crate::{
    backend::BackendBuilder,
    math::{bits_iter::BitsIter, types::*},
    register::{CReg, QReg},
};

#[derive(Clone, Debug)]
pub struct Sym<B: BackendBuilder> {
    backend_builder: B,
    m_op: MeasureOp,
    q_reg: QReg<B::Backend>,
    c_reg: CReg,
    q_ops: ExtOp,
}

impl<B: BackendBuilder + Clone> Sym<B> {
    pub fn new(int: Int<'_>, builder: B) -> Self {
        Self {
            backend_builder: builder.clone(),
            m_op: int.m_op,
            q_reg: QReg::with_builder(int.q_reg.len(), builder),
            c_reg: CReg::new(int.c_reg.len()),
            q_ops: int.q_ops,
        }
    }

    pub fn init(&mut self, int: Int<'_>) {
        if self.m_op != int.m_op
            || self.q_ops != int.q_ops
            || self.q_reg.num() != int.q_reg.len()
            || self.c_reg.num() != int.c_reg.len()
        {
            *self = Self::new(int, self.backend_builder.clone());
        }
    }

    pub fn reset(&mut self) {
        self.q_reg.reset(0);
        self.c_reg.reset(0);
    }

    pub fn finish(&mut self) -> &mut Self {
        for (op, sep) in self.q_ops.0.iter() {
            match *sep {
                Sep::Nop => {
                    self.q_reg.apply(op);
                }
                Sep::Measure(q_arg, c_arg) => {
                    self.q_reg.apply(op);

                    let mask = self.q_reg.measure_mask(q_arg);
                    let mut c_reg = self.c_reg.clone();
                    match self.m_op {
                        MeasureOp::Set => BitsIter::from(q_arg)
                            .zip(BitsIter::from(c_arg))
                            .for_each(|(q, c)| c_reg.set(mask.get() & q != 0, c)),
                        MeasureOp::Xor => BitsIter::from(q_arg)
                            .zip(BitsIter::from(c_arg))
                            .for_each(|(q, c)| c_reg.xor(mask.get() & q != 0, c)),
                    };
                    self.c_reg = c_reg;
                }
                Sep::IfBranch(c, v) => {
                    if self.c_reg.get_by_mask(c) == v {
                        self.q_reg.apply(op);
                    }
                }
                Sep::Reset(q) => {
                    self.q_reg.apply(op);
                    self.q_reg.reset_by_mask(q);
                }
            }
        }
        self.q_reg.apply(&self.q_ops.1);
        self
    }

    pub fn measure(&mut self, q_arg: N, c_arg: N) {
        let mask = self.q_reg.measure_mask(q_arg);

        match self.m_op {
            MeasureOp::Set => BitsIter::from(q_arg)
                .zip(BitsIter::from(c_arg))
                .for_each(|(q, c)| self.c_reg.set(mask.get() & q != 0, c)),
            MeasureOp::Xor => BitsIter::from(q_arg)
                .zip(BitsIter::from(c_arg))
                .for_each(|(q, c)| self.c_reg.xor(mask.get() & q != 0, c)),
        };
    }

    pub fn get_class(&self) -> CReg {
        self.c_reg.clone()
    }

    pub fn get_polar_wavefunction(&self) -> Vec<(R, R)> {
        self.q_reg.get_polar()
    }

    pub fn get_probabilities(&self) -> Vec<R> {
        self.q_reg.get_probabilities()
    }
}
