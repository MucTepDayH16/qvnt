use super::*;
use crate::{math::consts::PI, operator::single::rotate::rz};

pub fn qft(a_mask: N) -> MultiOp {
    let count = a_mask.count_ones() as usize;
    match count {
        0 => MultiOp::default(),
        1 => h::h(a_mask),
        _ => {
            let mut res = VecDeque::new();
            let mut vec = Vec::<usize>::with_capacity(count);

            for idx in 0..64 {
                let jdx = 1 << idx;
                if jdx & a_mask != 0 {
                    vec.push(jdx);
                }
            }

            for i in 0..(count - 1) {
                res.append(&mut h::h(vec[i]));
                res.extend((1..(count - i)).map(|j| {
                    rz(vec[i + j], PI * 0.5f64.powi(j as i32))
                        .and_then(|op| op.c(vec[i]))
                        .unwrap()
                }));
            }

            res.append(&mut h::h(vec[count - 1]).0);
            MultiOp(res)
        }
    }
}

pub fn qft_swapped(a_mask: N) -> MultiOp {
    let mut vec_mask = Vec::with_capacity(a_mask.count_ones() as N);
    let mut idx = 1;
    while idx <= a_mask {
        if idx & a_mask != 0 {
            vec_mask.push(idx);
        }
        idx <<= 1;
    }

    let mut swaps = MultiOp::default();
    let len = vec_mask.len();
    for i in 0..(len >> 1) {
        swaps *= crate::operator::single::swap::swap(vec_mask[i] | vec_mask[len - i - 1]).unwrap();
    }

    qft(a_mask) * swaps
}
