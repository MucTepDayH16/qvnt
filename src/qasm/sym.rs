use crate::{register::{CReg, QReg}, math::{N, bits_iter::BitsIter, R}};
use super::int::*;

#[derive(Clone, Debug)]
pub struct Sym {
    q_reg: QReg,
    c_reg: CReg,
    int: Box<Int>,
}

impl Sym {
    pub fn new(int: Int) -> Self {
        Self {
            q_reg: QReg::new(int.q_reg.len()).init_state(0),
            c_reg: CReg::new(int.c_reg.len()).init_state(0),
            int: Box::new(int),
        }
    }

    pub fn update(&mut self, int: &Int) {
        if int != &*self.int {
            *self = Self::new(int.clone());
        }
    }

    pub fn reset(&mut self) {
        self.q_reg.reset(0);
        self.c_reg.reset(0);
    }
    
    pub fn finish(&mut self) -> &mut Self {
        for (op, sep) in self.int.q_ops.0.iter() {
            match *sep {
                Sep::Nop => {
                    self.q_reg.apply(op);
                }
                Sep::Measure(q_arg, c_arg) => {
                    self.q_reg.apply(op);

                    let mask = self.q_reg.measure_mask(q_arg);
                    let mut c_reg = self.c_reg.clone();
                    match self.int.m_op {
                        MeasureOp::Set => BitsIter::from(q_arg)
                            .zip(BitsIter::from(c_arg))
                            .for_each(|(q, c)| c_reg.set(mask.get() & q != 0, c)),
                        MeasureOp::Xor => BitsIter::from(q_arg)
                            .zip(BitsIter::from(c_arg))
                            .for_each(|(q, c)| c_reg.xor(mask.get() & q != 0, c)),
                    };
                    self.c_reg = c_reg;
                },
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
        self.q_reg.apply(&self.int.q_ops.1);
        self
    }

    pub fn measure(&mut self, q_arg: N, c_arg: N) {
        let mask = self.q_reg.measure_mask(q_arg);

        match self.int.m_op {
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