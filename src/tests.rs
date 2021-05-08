use crate::{types::*, register::*, operator::*};

#[test]
fn ops() {
    let pend_ops = PendingOps::new()
        | Op::id()
        | Op::h(0b001, 0b010)
        | Op::x(0b011, 0b100)
        | Op::phi(vec![(0b001, 5.)], 0b000);

    assert_eq!(pend_ops.len(), 4);
}

#[test]
fn quantum_reg() {
    let mut reg = QReg::new(4).init_state(0b1110);
    let mask = 0b0110;

    let mut pend_ops = PendingOps::new()
        | Op::h_uc(0b1111)
        | Op::phi(vec![(0b1010, 0.25 * PI), (0b1100, 0.5 * PI)], 0b0000)
        ;
    reg.eval(&pend_ops);
    pend_ops.clear();

    println!("{:?}", reg);

    assert_eq!(pend_ops.len(), 0);
    assert_eq!(reg.measure(mask) & !mask, 0);
}

#[test]
fn tensor() {
    let mut reg1 = QReg::new(2).init_state(0b01);
    let mut reg2 = QReg::new(1).init_state(0b1);

    let mut pend_ops = PendingOps::new() | Op::h_uc(0b01);

    reg1.eval(&pend_ops);
    reg2.eval(&pend_ops);

    let test_prob = (reg1 * reg2).get_probabilities();
    let true_prob = vec![0.25, 0.25, 0., 0., 0.25, 0.25, 0., 0.];

    let eps = 1e-9;
    assert!(
        test_prob.into_iter().zip(true_prob.into_iter())
            .all(|(a, b)| (a - b).abs() < eps)
    );
}

#[test]
fn aliases_concat() {
    let alias1 = String::from("wat");
    let reg1 = QReg::new(3)
        .init_state(0)
        .set_alias_str(alias1);

    let alias2 = String::from("aka");
    let reg2 = QReg::new(3)
        .init_state(0)
        .set_alias_str(alias2);

    let alias = String::from("wataka");
    let reg = QReg::new(6)
        .init_state(0)
        .set_alias_str(alias);

    assert_eq!(format!("{}", reg), format!("{}", reg1 * reg2));
}

#[test]
fn virtual_regs() {
    let mut reg = QReg::new(0);

    //  qreg    x[3];
    reg *= QReg::new(3).set_alias_char('x');
    //  qreg    a[2];
    reg *= QReg::new(2).set_alias_char('a');

    let r = reg.get_vreg();
    let m = reg.get_vreg_by_mask(0b01101).unwrap();
    let x = reg.get_vreg_by_char('x').unwrap();
    let a = reg.get_vreg_by_char('a').unwrap();

    assert_eq!(r[0],    0b00001);
    assert_eq!(r[1],    0b00010);
    assert_eq!(r[2],    0b00100);
    assert_eq!(r[3],    0b01000);
    assert_eq!(r[4],    0b10000);
    assert_eq!(r[..],   0b11111);

    assert_eq!(m[0],    0b00001);
    assert_eq!(m[1],    0b00100);
    assert_eq!(m[2],    0b01000);
    assert_eq!(m[..],   0b01101);

    assert_eq!(x[0],    0b00001);
    assert_eq!(x[1],    0b00010);
    assert_eq!(x[2],    0b00100);
    assert_eq!(x[..],   0b00111);

    assert_eq!(a[0],    0b01000);
    assert_eq!(a[1],    0b10000);
    assert_eq!(a[..],   0b11000);
}

#[test]
fn qft() {
    let mut pend_ops = Op::qft_no_swap(0b1111);

    println!("{:?}", pend_ops);
}