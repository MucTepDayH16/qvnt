use super::*;

macro_rules! gate {
    (any $op:ident $regs:expr , $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg);
        if regs == 0 {
            Err(Error::WrongRegNumber(stringify!($op).to_string(), 0))
        } else if $args.len() != 0 {
            Err(Error::WrongArgNumber(stringify!($op).to_string(), $args.len()))
        } else {
            Ok(op::$op(regs))
        }
    }};
    (2 $op:ident $regs:expr , $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg);
        if crate::math::count_bits(regs) != 2 {
            Err(Error::WrongRegNumber(stringify!($op).to_string(), crate::math::count_bits(regs)))
        } else if $args.len() != 0 {
            Err(Error::WrongArgNumber(stringify!($op).to_string(), $args.len()))
        } else {
            Ok(op::$op(regs))
        }
    }};
    (r($num:expr) $op:ident $regs:expr , $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg);
        if crate::math::count_bits(regs) != $num {
            Err(Error::WrongRegNumber(stringify!($op).to_string(), crate::math::count_bits(regs)))
        } else if $args.len() != 1 {
            Err(Error::WrongArgNumber(stringify!($op).to_string(), $args.len()))
        } else {
            Ok(op::$op($args[0], regs))
        }
    }};
    (u1 $regs:expr , $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg);
        if crate::math::count_bits(regs) != 1 {
            Err(Error::WrongRegNumber(stringify!($op).to_string(), crate::math::count_bits(regs)))
        } else if $args.len() != 1 {
            Err(Error::WrongArgNumber(stringify!($op).to_string(), $args.len()))
        } else {
            Ok(op::u1($args[0], regs))
        }
    }};
    (u2 $regs:expr , $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg);
        if crate::math::count_bits(regs) != 1 {
            Err(Error::WrongRegNumber(stringify!($op).to_string(), crate::math::count_bits(regs)))
        } else if $args.len() != 2 {
            Err(Error::WrongArgNumber(stringify!($op).to_string(), $args.len()))
        } else {
            Ok(op::u2($args[0], $args[1], regs))
        }
    }};
    (u3 $regs:expr , $args:expr) => {{
        let regs = $regs.into_iter().fold(0, |acc, reg| acc | reg);
        if crate::math::count_bits(regs) != 1 {
            Err(Error::WrongRegNumber(stringify!($op).to_string(), crate::math::count_bits(regs)))
        } else if $args.len() != 3 {
            Err(Error::WrongArgNumber(stringify!($op).to_string(), $args.len()))
        } else {
            Ok(op::u3($args[0], $args[1], $args[2], regs))
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
                .map_err(|err| match err {
                    Error::WrongRegNumber(name, num) =>
                        Error::WrongRegNumber("c".to_string() + &name, 1 + num),
                    Error::WrongArgNumber(name, num) =>
                        Error::WrongArgNumber("c".to_string() + &name, num),
                    Error::UnknownGate(name) =>
                        Error::UnknownGate("c".to_string() + &name),
                    e => e,
                })
        },
        "x"             => gate!(any x regs , args),
        "y"             => gate!(any y regs , args),
        "z"             => gate!(any z regs , args),
        "s"             => gate!(any s regs , args),
        "t"             => gate!(any t regs , args),

        "h"             => gate!(any h regs , args),

        "rx"            => gate!(r(1) rx regs , args),
        "ry"            => gate!(r(1) ry regs , args),
        "rz"            => gate!(r(1) rz regs , args),

        "rxx"           => gate!(r(2) rxx regs , args),
        "ryy"           => gate!(r(2) ryy regs , args),
        "rzz"           => gate!(r(2) rzz regs , args),

        "swap"          => gate!(2 swap regs , args),
        "sqrt_swap"     => gate!(2 sqrt_swap regs , args),
        "i_swap"        => gate!(2 i_swap regs , args),
        "sqrt_i_swap"   => gate!(2 sqrt_i_swap regs , args),

        "u1"            => gate!(u1 regs , args),
        "u2"            => gate!(u2 regs , args),
        "u3"            => gate!(u3 regs , args),

        _               => Err(Error::UnknownGate(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name() {
        assert_eq!(
            Ok(op::x(0b111)),
            process("x".to_string(), vec![0b111], vec![])
        );
        assert_eq!(
            Err(Error::WrongArgNumber("x".to_string(), 1)),
            process("x".to_string(), vec![0b111], vec![1.0])
        );
        assert_eq!(
            Ok(op::x(0b011).c(0b100)),
            process("cx".to_string(), vec![0b100, 0b010, 0b001], vec![])
        );
        assert_eq!(
            Err(Error::WrongRegNumber("cx".to_string(), 1)),
            process("cx".to_string(), vec![0b100], vec![])
        );
        assert_eq!(
            Ok(op::x(0b001).c(0b110)),
            process("ccx".to_string(), vec![0b100, 0b010, 0b001], vec![])
        );
        assert_eq!(
            Err(Error::WrongArgNumber("ccx".to_string(), 1)),
            process("ccx".to_string(), vec![0b100, 0b010, 0b001], vec![1.0])
        );

        assert_eq!(
            Ok(op::rx(1.0, 0b100)),
            process("rx".to_string(), vec![0b100], vec![1.0])
        );
        assert_eq!(
            Err(Error::WrongRegNumber("rx".to_string(), 2)),
            process("rx".to_string(), vec![0b101], vec![1.0])
        );
        assert_eq!(
            Err(Error::WrongArgNumber("rx".to_string(), 0)),
            process("rx".to_string(), vec![0b100], vec![])
        );

        assert_eq!(
            Ok(op::rxx(1.0, 0b101)),
            process("rxx".to_string(), vec![0b101], vec![1.0])
        );
        assert_eq!(
            Err(Error::WrongRegNumber("rxx".to_string(), 1)),
            process("rxx".to_string(), vec![0b100], vec![1.0])
        );
        assert_eq!(
            Err(Error::WrongArgNumber("rxx".to_string(), 2)),
            process("rxx".to_string(), vec![0b101], vec![2.0, 1.0])
        );

        assert_eq!(
            Ok(op::swap(0b101)),
            process("swap".to_string(), vec![0b101], vec![])
        );
        assert_eq!(
            Err(Error::WrongRegNumber("swap".to_string(), 3)),
            process("swap".to_string(), vec![0b111], vec![1.0])
        );
        assert_eq!(
            Err(Error::WrongArgNumber("swap".to_string(), 1)),
            process("swap".to_string(), vec![0b101], vec![1.0])
        );

        assert_eq!(
            Ok(op::u1(1.0, 0b001)),
            process("u1".to_string(), vec![0b001], vec![1.0])
        );
        assert_eq!(
            Ok(op::u2(1.0, 2.0,0b001)),
            process("u2".to_string(), vec![0b001], vec![1.0, 2.0])
        );
        assert_eq!(
            Ok(op::u3(1.0, 2.0, 3.0, 0b001)),
            process("u3".to_string(), vec![0b001], vec![1.0, 2.0, 3.0])
        );
    }
}