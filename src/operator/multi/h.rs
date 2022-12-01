use super::*;
use crate::{math::count_bits, operator::atomic::*};

#[inline(always)]
fn h1(a_mask: Mask) -> SingleOp {
    H1::new(a_mask).into()
}

#[inline(always)]
fn h2(a_mask: Mask, b_mask: Mask) -> SingleOp {
    H2::new(a_mask, b_mask).into()
}

pub fn h(a_mask: Mask) -> MultiOp {
    let count = count_bits(a_mask);

    match count {
        0 => MultiOp::default(),
        1 => h1(a_mask).into(),
        _ => {
            let mut res = MultiOp(VecDeque::with_capacity((count + 1) >> 1));
            let mut idx = (1, 0);
            let mut is_first = true;

            while idx.0 <= a_mask {
                if idx.0 & a_mask != 0 {
                    if is_first {
                        idx.1 = idx.0;
                        is_first = false;
                    } else {
                        res.0.push_back(h2(idx.0, idx.1));
                        is_first = true;
                    }
                }
                idx.0 <<= 1;
            }

            if !is_first {
                res.0.push_back(h1(idx.1));
            }

            res
        }
    }
}
