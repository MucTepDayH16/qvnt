use super::int::*;
use crate::{
    math::{bits_iter::BitsIter, N, R},
    register::{CReg, QReg},
};

#[derive(Clone, Debug)]
pub struct Sym {
    m_op: MeasureOp,
    q_reg: QReg,
    c_reg: CReg,
    q_ops: ExtOp,
}

impl Sym {
    pub fn new<'t>(int: Int<'t>) -> Self {
        Self {
            m_op: int.m_op,
            q_reg: QReg::new(int.q_reg.len()),
            c_reg: CReg::new(int.c_reg.len()),
            q_ops: int.q_ops.clone(),
        }
    }

    pub fn init<'t>(&mut self, int: Int<'t>) {
        if self.m_op != int.m_op
            || self.q_ops != int.q_ops
            || self.q_reg.num() != int.q_reg.len()
            || self.c_reg.num() != int.c_reg.len()
        {
            *self = Self::new(int);
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
