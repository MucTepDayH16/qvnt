use crate::{types::*, register::*, operator::*};

#[test]
fn ops() {
    let pend_ops =
        Op::id() *
        Op::h(0b001, 0b010) *
        Op::x(0b011, 0b100) *
        Op::phi(vec![(0b001, 5.)], 0b000);

    assert_eq!(pend_ops.len(), 3);
}

#[test]
fn quantum_reg() {
    let mut reg = QReg::new(4).init_state(0b1100);
    let mask = 0b0110;

    let mut op =
        Op::h(0b1111, 0b0000)
        * Op::h(0b0011, 0b1000)
        * Op::swap(0b0001, 0b0100, 0b0010)
        ;

    reg.apply(&op);

    println!("{:?}", op);
    println!("{:?}", reg);

    op.clear();
    assert_eq!(op.len(), 0);
    assert_eq!(reg.measure(mask) & !mask, 0);
}

#[test]
fn tensor() {
    let mut reg1 = QReg::new(2).init_state(0b01);
    let mut reg2 = QReg::new(1).init_state(0b1);

    let mut pend_ops = Op::h(0b01, 0);

    reg1.apply(&pend_ops);
    reg2.apply(&pend_ops);

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

#[test]
fn operator_from_matrix() {
    use crate::math::*;

    assert!(is_unitary_m1(&[
        C::one(), C::zero(),
        C::zero(), C::one()
    ]));

    assert!(!is_unitary_m1(&[
        C::one(), C::one(),
        C::one(), -C::one()
    ]));
    assert!(is_scaled_unitary_m1(&[
        C::one(), C::one(),
        C::one(), -C::one()
    ]));

    assert!(!is_unitary_m1(&[
        C::one(), 2.0 * C::one(),
        C::one(), -C::one()
    ]));
    assert!(!is_scaled_unitary_m1(&[
        C::one(), 2.0 * C::one(),
        C::one(), -C::one()
    ]));

    assert!(is_unitary_m1(&[
        C::one() / SQRT_2, C::one() / SQRT_2,
        C::one() / SQRT_2, -C::one() / SQRT_2
    ]));
    assert!(is_hermitian_m1(&[
        C::one() / SQRT_2, C::one() / SQRT_2,
        C::one() / SQRT_2, -C::one() / SQRT_2
    ]));

    let angle: R = FRAC_PI_6;
    let mut matrix = [C::zero(); 4];
    matrix[0] = C::new(angle.cos(), 0.0);
    matrix[1] = C::new(0.0, -angle.sin());
    matrix[2] = matrix[1];
    matrix[3] = matrix[0];
    println!("{:?}", Op::uni_1x1(matrix, 0b001, 0));
    println!("{:?}", Op::if_b_then_u1_else_u0(matrix, matrix, 0b0001, 0b1000, 0))
}

#[test]
fn rotate_operators() {
    const PHASE: R = 1.1;

    println!("{:?}", (PHASE * 0.5).sin_cos());

    //  rx
    let mut reg0 = QReg::new(1).init_state(0b0);
    reg0.apply(&Op::rx(PHASE, 0b1, 0));
    let mut reg1 = QReg::new(1).init_state(0b1);
    reg1.apply(&Op::rx(PHASE, 0b1, 0));
    println!("{:?}\n{:?}\n", reg0, reg1);

    //  ry
    let mut reg0 = QReg::new(1).init_state(0b0);
    reg0.apply(&Op::ry(PHASE, 0b1, 0));
    let mut reg1 = QReg::new(1).init_state(0b1);
    reg1.apply(&Op::ry(PHASE, 0b1, 0));
    println!("{:?}\n{:?}\n", reg0, reg1);


    //  rz
    let mut reg0 = QReg::new(1).init_state(0b0);
    reg0.apply(&Op::rz(PHASE, 0b1, 0));
    let mut reg1 = QReg::new(1).init_state(0b1);
    reg1.apply(&Op::rz(PHASE, 0b1, 0));
    println!("{:?}\n{:?}\n", reg0, reg1);

    //  rxx
    let op = Op::rxx(PHASE, 0b11, 0);

    let mut reg0 = QReg::new(2).init_state(0b00);
    reg0.apply(&op);
    let mut reg1 = QReg::new(2).init_state(0b01);
    reg1.apply(&op);
    let mut reg2 = QReg::new(2).init_state(0b10);
    reg2.apply(&op);
    let mut reg3 = QReg::new(2).init_state(0b11);
    reg3.apply(&op);
    println!("{:?}\n{:?}\n{:?}\n{:?}\n", reg0, reg1, reg2, reg3);

    //  ryy
    let op = Op::ryy(PHASE, 0b11, 0);

    let mut reg0 = QReg::new(2).init_state(0b00);
    reg0.apply(&op);
    let mut reg1 = QReg::new(2).init_state(0b01);
    reg1.apply(&op);
    let mut reg2 = QReg::new(2).init_state(0b10);
    reg2.apply(&op);
    let mut reg3 = QReg::new(2).init_state(0b11);
    reg3.apply(&op);
    println!("{:?}\n{:?}\n{:?}\n{:?}\n", reg0, reg1, reg2, reg3);

    //  rzz
    let op = Op::rzz(PHASE, 0b11, 0);

    let mut reg0 = QReg::new(2).init_state(0b00);
    reg0.apply(&op);
    let mut reg1 = QReg::new(2).init_state(0b01);
    reg1.apply(&op);
    let mut reg2 = QReg::new(2).init_state(0b10);
    reg2.apply(&op);
    let mut reg3 = QReg::new(2).init_state(0b11);
    reg3.apply(&op);
    println!("{:?}\n{:?}\n{:?}\n{:?}\n", reg0, reg1, reg2, reg3);
}