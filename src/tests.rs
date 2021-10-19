use crate::{register::*, operator::*};

#[test]
fn quantum_reg() {
    let mut reg = QReg::new(4).init_state(0b1100);
    let mask = 0b0110;

    let mut operator =
        op::h(0b1111)
        * op::h(0b0011).c(0b1000)
        * op::swap(0b1001).c(0b0010)
        ;

    reg.apply(&operator);

    println!("{:?}", operator);
    println!("{:?}", reg);

    operator.clear();
    assert_eq!(operator.len(), 0);
    assert_eq!(reg.measure_mask(mask) & !mask, 0);
}

#[test]
fn tensor() {
    let mut reg1 = QReg::new(2).init_state(0b01);
    let mut reg2 = QReg::new(1).init_state(0b1);

    let mut pend_ops = op::h(0b01);

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
fn virtual_regs() {
    let mut reg = QReg::new(0);

    //  qreg    x[3];
    reg *= QReg::new(3);
    //  qreg    a[2];
    reg *= QReg::new(2);

    let r = reg.get_vreg();
    let m = reg.get_vreg_by_mask(0b01101).unwrap();
    let x = reg.get_vreg_by_mask(0b00111).unwrap();
    let y = reg.get_vreg_by_mask(0b11000).unwrap();

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

    assert_eq!(y[0],    0b01000);
    assert_eq!(y[1],    0b10000);
    assert_eq!(y[..],   0b11000);
}