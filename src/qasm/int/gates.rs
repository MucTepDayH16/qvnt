use super::*;

macro_rules! gate {
    ($name:expr, any, $op:ident, $regs:expr, $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg,);
        if regs == 0 {
            Err(Error::WrongRegNumber($name, 0,),)
        } else if $args.len() != 0 {
            Err(Error::WrongArgNumber($name, $args.len(),),)
        } else {
            Ok(op::$op(regs,),)
        }
    }};
    ($name:expr, dgr, $op:ident, $regs:expr, $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg,);
        if regs == 0 {
            Err(Error::WrongRegNumber($name, 0,),)
        } else if $args.len() != 0 {
            Err(Error::WrongArgNumber($name, $args.len(),),)
        } else {
            Ok(op::$op(regs,),)
        }
    }};
    ($name:expr, 2, $op:ident, $regs:expr, $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg,);
        if crate::math::count_bits(regs,) != 2 {
            Err(Error::WrongRegNumber($name, crate::math::count_bits(regs,),),)
        } else if $args.len() != 0 {
            Err(Error::WrongArgNumber($name, $args.len(),),)
        } else {
            Ok(op::$op(regs,),)
        }
    }};
    ($name:expr, r($num:expr), $op:ident, $regs:expr, $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg,);
        if crate::math::count_bits(regs,) != $num {
            Err(Error::WrongRegNumber($name, crate::math::count_bits(regs,),),)
        } else if $args.len() != 1 {
            Err(Error::WrongArgNumber($name, $args.len(),),)
        } else {
            Ok(op::$op($args[0], regs,),)
        }
    }};
    ($name:expr, u1, $regs:expr, $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg,);
        if crate::math::count_bits(regs,) != 1 {
            Err(Error::WrongRegNumber($name, crate::math::count_bits(regs,),),)
        } else if $args.len() != 1 {
            Err(Error::WrongArgNumber($name, $args.len(),),)
        } else {
            Ok(op::u1($args[0], regs,),)
        }
    }};
    ($name:expr, u2, $regs:expr, $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg,);
        if crate::math::count_bits(regs,) != 1 {
            Err(Error::WrongRegNumber($name, crate::math::count_bits(regs,),),)
        } else if $args.len() != 2 {
            Err(Error::WrongArgNumber($name, $args.len(),),)
        } else {
            Ok(op::u2($args[0], $args[1], regs,),)
        }
    }};
    ($name:expr, u3, $regs:expr, $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg,);
        if crate::math::count_bits(regs,) != 1 {
            Err(Error::WrongRegNumber($name, crate::math::count_bits(regs,),),)
        } else if $args.len() != 3 {
            Err(Error::WrongArgNumber($name, $args.len(),),)
        } else {
            Ok(op::u3($args[0], $args[1], $args[2], regs,),)
        }
    }};
}

pub(crate) fn process<'t,>(name: &'t str, regs: Vec<N,>, args: Vec<R,>,) -> Result<'t, MultiOp,> {
    match &*name {
        s if matches!(&s[..1], "c" | "C") => {
            let (&ctrl, regs,) = regs.split_first().ok_or(Error::WrongRegNumber(name, 0,),)?;

            match process(&name[1..], regs.into(), args,) {
                Ok(op,) => {
                    let act = op.act_on();
                    op.c(ctrl,).ok_or(Error::InvalidControlMask(ctrl, act,),)
                }
                Err(err,) => Err(match err {
                    Error::WrongRegNumber(_, num,) => Error::WrongRegNumber(name, 1 + num,),
                    Error::WrongArgNumber(_, num,) => Error::WrongArgNumber(name, num,),
                    Error::UnknownGate(_,) => Error::UnknownGate(name,),
                    e => e,
                },),
            }
        }
        "x" | "X" => gate!(name, any, x, regs, args),
        "y" | "Y" => gate!(name, any, y, regs, args),
        "z" | "Z" => gate!(name, any, z, regs, args),
        "s" | "S" => gate!(name, any, s, regs, args),
        "sdg" | "SDG" => gate!(name, dgr, s, regs, args),
        "t" | "T" => gate!(name, any, t, regs, args),
        "tdg" | "TDG" => gate!(name, dgr, t, regs, args),

        "h" | "H" => gate!(name, any, h, regs, args),
        "qft" | "QFT" => gate!(name, any, qft, regs, args),

        "rx" | "RX" => gate!(name, r(1), rx, regs, args),
        "ry" | "RY" => gate!(name, r(1), ry, regs, args),
        "rz" | "RZ" => gate!(name, r(1), rz, regs, args),

        "rxx" | "RXX" => gate!(name, r(2), rxx, regs, args),
        "ryy" | "RYY" => gate!(name, r(2), ryy, regs, args),
        "rzz" | "RZZ" => gate!(name, r(2), rzz, regs, args),

        "swap" | "SWAP" => gate!(name, 2, swap, regs, args),
        "sqrt_swap" | "SQRT_SWAP" => gate!(name, 2, sqrt_swap, regs, args),
        "i_swap" | "I_SWAP" => gate!(name, 2, i_swap, regs, args),
        "sqrt_i_swap" | "SQRT_I_SWAP" => gate!(name, 2, sqrt_i_swap, regs, args),

        "u1" | "U1" => gate!(name, u1, regs, args),
        "u2" | "U2" => gate!(name, u2, regs, args),
        "u3" | "U3" => gate!(name, u3, regs, args),

        _ => Err(Error::UnknownGate(name,),),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_process_x() {
        assert_eq!(process("x", vec![0b111], vec![]), Ok(op::x(0b111)),);
        assert_eq!(
            process("x", vec![0b111], vec![1.0]),
            Err(Error::WrongArgNumber("x", 1)),
        );
    }

    #[test]
    fn try_process_cx() {
        assert_eq!(
            process("cx", vec![0b100, 0b010, 0b001], vec![]),
            Ok(op::x(0b011).c(0b100).unwrap()),
        );
        assert_eq!(
            process("cx", vec![0b100], vec![]),
            Err(Error::WrongRegNumber("cx", 1)),
        );
        assert_eq!(
            process("cx", vec![0b100, 0b010, 0b001], vec![1.0]),
            Err(Error::WrongArgNumber("cx", 1)),
        );
    }

    #[test]
    fn try_process_ccx() {
        assert_eq!(
            process("ccx", vec![0b100, 0b010, 0b001], vec![]),
            Ok(op::x(0b001).c(0b110).unwrap()),
        );
        assert_eq!(
            process("ccx", vec![0b100], vec![]),
            Err(Error::WrongRegNumber("ccx", 1)),
        );
        assert_eq!(
            process("ccx", vec![0b100, 0b010, 0b001], vec![1.0]),
            Err(Error::WrongArgNumber("ccx", 1)),
        );
    }

    #[test]
    fn try_process_rx() {
        assert_eq!(
            process("rx", vec![0b100], vec![1.0]),
            Ok(op::rx(1.0, 0b100)),
        );
        assert_eq!(
            process("rx", vec![0b101], vec![1.0]),
            Err(Error::WrongRegNumber("rx", 2)),
        );
        assert_eq!(
            process("rx", vec![0b100], vec![]),
            Err(Error::WrongArgNumber("rx", 0)),
        );
    }

    #[test]
    fn try_process_rxx() {
        assert_eq!(
            process("rxx", vec![0b101], vec![1.0]),
            Ok(op::rxx(1.0, 0b101)),
        );
        assert_eq!(
            process("rxx", vec![0b100], vec![1.0]),
            Err(Error::WrongRegNumber("rxx", 1)),
        );
        assert_eq!(
            process("rxx", vec![0b101], vec![2.0, 1.0]),
            Err(Error::WrongArgNumber("rxx", 2)),
        );
    }

    #[test]
    fn try_process_swap() {
        assert_eq!(process("swap", vec![0b101], vec![]), Ok(op::swap(0b101)),);
        assert_eq!(
            process("swap", vec![0b111], vec![1.0]),
            Err(Error::WrongRegNumber("swap", 3)),
        );
        assert_eq!(
            process("swap", vec![0b101], vec![1.0]),
            Err(Error::WrongArgNumber("swap", 1)),
        );
    }

    #[test]
    fn try_process_unitary() {
        assert_eq!(
            process("u1", vec![0b001], vec![1.0]),
            Ok(op::u1(1.0, 0b001)),
        );
        assert_eq!(
            process("u2", vec![0b001], vec![1.0, 2.0]),
            Ok(op::u2(1.0, 2.0, 0b001)),
        );
        assert_eq!(
            process("u3", vec![0b001], vec![1.0, 2.0, 3.0]),
            Ok(op::u3(1.0, 2.0, 3.0, 0b001)),
        );
    }

    #[test]
    fn try_process_any() {
        assert_eq!(process("x", vec![0b001, 0b100], vec![]), Ok(op::x(0b101)),);
        assert_eq!(process("y", vec![0b11], vec![]), Ok(op::y(0b11)),);
        assert_eq!(
            process("ch", vec![0b100, 0b010, 0b001], vec![]),
            Ok(op::h(0b011).c(0b100).unwrap()),
        );
        assert_eq!(
            process("swap", vec![0b100, 0b010], vec![]),
            Ok(op::swap(0b110)),
        );
        assert_eq!(
            process("swap", vec![0b001], vec![]),
            Err(Error::WrongRegNumber("swap", 1)),
        );
    }
}
