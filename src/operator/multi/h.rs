use super::*;
use crate::operator::atomic;

#[inline(always)]
fn h1(a_mask: N) -> SingleOp {
    atomic::h1::Op::new(a_mask).into()
}

#[inline(always)]
fn h2(a_mask: N, b_mask: N) -> SingleOp {
    atomic::h2::Op::new(a_mask, b_mask).into()
}

pub fn h(a_mask: N) -> MultiOp {
    let count = a_mask.count_ones() as N;

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
