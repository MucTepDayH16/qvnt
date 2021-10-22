use super::*;

macro_rules! gate {
    (any $op:ident, $regs:expr) => {
        Ok(op::$op($regs.into_iter().fold(0, |acc, reg| acc | reg)))
    };
    (2 $op:ident, $regs:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg);
        if crate::math::count_bits(regs) == 2 {
            Ok(op::$op(regs))
        } else {
            Err(Error::WrongRegNumber(stringify!($op).to_string(), crate::math::count_bits(regs)))
        }
    }};
}

pub(crate) fn process(name: String, regs: Vec<N>, args: Vec<R>) -> Result<MultiOp> {
    match name.as_str() {
        s if s.chars().next() == Some('c') => {
            let mut name = name.chars();
            name.next();
            process(name.collect(), Vec::from(&regs[1..]), args)
                .map(|op| op.c(regs[0]))
        },
        "x"             => gate!(any x, regs),
        "y"             => gate!(any y, regs),
        "z"             => gate!(any z, regs),
        "s"             => gate!(any s, regs),
        "t"             => gate!(any t, regs),
        "h"             => gate!(any h, regs),
        "swap"          => gate!(2 swap, regs),
        "sqrt_swap"     => gate!(2 sqrt_swap, regs),
        "i_swap"        => gate!(2 i_swap, regs),
        "sqrt_i_swap"   => gate!(2 sqrt_i_swap, regs),
        _               => Err(Error::UnknownGate(name.clone()))
    }
}