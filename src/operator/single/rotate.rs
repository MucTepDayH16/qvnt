use super::*;

#[inline(always)]
pub fn rx(a_mask: Mask, phase: R) -> Option<SingleOp> {
    single_op_checked!(RX::new(a_mask, phase))
}

#[inline(always)]
pub fn rxx(ab_mask: Mask, phase: R) -> Option<SingleOp> {
    single_op_checked!(RXX::new(ab_mask, phase))
}

#[inline(always)]
pub fn ry(a_mask: Mask, phase: R) -> Option<SingleOp> {
    single_op_checked!(RY::new(a_mask, phase))
}

#[inline(always)]
pub fn ryy(ab_mask: Mask, phase: R) -> Option<SingleOp> {
    single_op_checked!(RYY::new(ab_mask, phase))
}

#[inline(always)]
pub fn rz(a_mask: Mask, phase: R) -> Option<SingleOp> {
    single_op_checked!(RZ::new(a_mask, phase))
}

#[inline(always)]
pub fn rzz(ab_mask: Mask, phase: R) -> Option<SingleOp> {
    single_op_checked!(RZZ::new(ab_mask, phase))
}
