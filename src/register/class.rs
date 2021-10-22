use {
    std::{
        fmt,
        mem::take,
        ops::{Mul, MulAssign},
    },

    crate::{
        math::{C, R, N},
    },
};

#[derive(Debug, Default)]
pub struct Reg {
    value: N,
    q_num: N,
    q_mask: N,
}

impl Reg {
    pub fn new(q_num: N) -> Self {
        let q_mask = 1_usize.wrapping_shl(q_num as u32).wrapping_add(!0_usize);

        Self{ value: 0, q_num, q_mask }
    }

    pub fn init_state(self, i_state: N) -> Self {
        Self{ value: i_state & self.q_mask, ..self }
    }

    fn tensor_prod(self, other: Self) -> Self {
        let shift = (0 as u8, self.q_num as u8);
        Self::new(self.q_num + other.q_num)
            .init_state((self.value << shift.0) | (other.value << shift.1))
    }
}

impl Mul for Reg {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.tensor_prod(other)
    }
}

impl MulAssign for Reg {
    fn mul_assign(&mut self, rhs: Self) {
        *self = take(self).tensor_prod(rhs);
    }
}