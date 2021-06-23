use crate::{types::*, register::*, operator::*};

#[test]
fn ops() {
    let pend_ops =
        Op::id() *
        Op::h(0b001).c(0b010) *
        Op::x(0b011).c(0b100) *
        Op::phi(vec![(5.0, 0b001)]);

    assert_eq!(pend_ops.len(), 3);
}

#[test]
fn quantum_reg() {
    let mut reg = QReg::new(4).init_state(0b1100);
    let mask = 0b0110;

    let mut op =
        Op::h(0b1111)
        * Op::h(0b0011).c(0b1000)
        * Op::swap(0b1001).c(0b0010)
        ;

    reg.apply(&op);

    println!("{:?}", op);
    println!("{:?}", reg);

    op.clear();
    assert_eq!(op.len(), 0);
    assert_eq!(reg.measure_mask(mask) & !mask, 0);
}

#[test]
fn tensor() {
    let mut reg1 = QReg::new(2).init_state(0b01);
    let mut reg2 = QReg::new(1).init_state(0b1);

    let mut pend_ops = Op::h(0b01);

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
    reg *= QReg::new(2).set_alias_char('y');

    let r = reg.get_vreg();
    let m = reg.get_vreg_by_mask(0b01101).unwrap();
    let x = reg.get_vreg_by_char('x').unwrap();
    let y = reg.get_vreg_by_char('y').unwrap();

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

#[test]
fn qft() {
    let mut pend_ops = Op::qft(0b1111);

    println!("{:?}", pend_ops);
}

#[test]
fn operator_from_matrix() {
    use crate::math::*;

    assert!(is_unitary_m1(&[
        C_ONE, C_ZERO,
        C_ZERO, C_ONE
    ]));

    assert!(!is_unitary_m1(&[
        C_ONE, C_ONE,
        C_ONE, -C_ONE
    ]));
    assert!(is_scaled_unitary_m1(&[
        C_ONE, C_ONE,
        C_ONE, -C_ONE
    ]));

    assert!(!is_unitary_m1(&[
        C_ONE, 2.0 * C_ONE,
        C_ONE, -C_ONE
    ]));
    assert!(!is_scaled_unitary_m1(&[
        C_ONE, 2.0 * C_ONE,
        C_ONE, -C_ONE
    ]));

    assert!(is_unitary_m1(&[
        C_ONE / SQRT_2, C_ONE / SQRT_2,
        C_ONE / SQRT_2, -C_ONE / SQRT_2
    ]));
    assert!(is_hermitian_m1(&[
        C_ONE / SQRT_2, C_ONE / SQRT_2,
        C_ONE / SQRT_2, -C_ONE / SQRT_2
    ]));

    let angle: R = FRAC_PI_6;
    let mut matrix = [C_ZERO; 4];
    matrix[0] = C::new(angle.cos(), 0.0);
    matrix[1] = C::new(0.0, -angle.sin());
    matrix[2] = matrix[1];
    matrix[3] = matrix[0];
    println!("{:?}", Op::uni_1x1(matrix, 0b001));
    println!("{:?}", Op::if_b_then_u1_else_u0(matrix, matrix, 0b0001, 0b1000))
}

fn get_matrix_2x2(ops: &Op) -> [[C; 2]; 2] {
    let mut matrix = [[C::from(0.); 2]; 2];

    for b in 0..2 {
        let mut reg = QReg::new(1).init_state(b);
        reg.apply(ops);
        matrix[0][b] = reg.psi[0];
        matrix[1][b] = reg.psi[1];
    }

    matrix
}

fn get_matrix_4x4(ops: &Op) -> [[C; 4]; 4] {
    let mut matrix = [[C::from(0.); 4]; 4];

    for b in 0..4 {
        let mut reg = QReg::new(2).init_state(b);
        reg.apply(ops);
        matrix[0][b] = reg.psi[0];
        matrix[1][b] = reg.psi[1];
        matrix[2][b] = reg.psi[2];
        matrix[3][b] = reg.psi[3];
    }

    matrix
}

#[test]
fn simple_operators() {
    const SQRT_1_2: R = SQRT_2 * 0.5;
    const _1: C = C{ re: 1.0, im: 0.0 };
    const _0: C = C{ re: 0.0, im: 0.0 };
    const _i: C = C{ re: 0.0, im: 1.0 };

    //x
    assert_eq!(
        Op::x(0b1).matrix_t::<2>(),
        [   [_0, _1],
            [_1, _0]]
    );
    //y
    assert_eq!(
        Op::y(0b1).matrix_t(),
        [   [_0, _i],
            [-_i, _0]]
    );
    //z
    assert_eq!(
        Op::z(0b1).matrix_t::<2>(),
        [   [_1, _0],
            [_0, -_1]]
    );
    //s
    assert_eq!(
        Op::s(0b1).matrix_t::<2>(),
        [   [_1, _0],
            [_0, _i]]
    );
    //t
    assert_eq!(
        Op::t(0b1).matrix_t::<2>(),
        [   [_1, _0                             ],
            [_0, SQRT_1_2 * (_1 + _i) ]]
    );

    //swap
    assert_eq!(
        Op::swap(0b11).matrix_t::<4>(),
        [   [_1, _0, _0, _0],
            [_0, _0, _1, _0],
            [_0, _1, _0, _0],
            [_0, _0, _0, _1]]
    );
    //i_swap
    assert_eq!(
        Op::i_swap(0b11).matrix_t::<4>(),
        [   [_1, _0, _0, _0],
            [_0, _0, _i, _0],
            [_0, _i, _0, _0],
            [_0, _0, _0, _1]]
    );
    //sqrt_swap
    assert_eq!(
        Op::sqrt_swap(0b11).matrix_t::<4>(),
        [   [_1, _0,                _0,                 _0],
            [_0, 0.5 * (_1 + _i),   0.5 * (_1 - _i),    _0],
            [_0, 0.5 * (_1 - _i),   0.5 * (_1 + _i),    _0],
            [_0, _0,                _0,                 _1]]
    );
    //sqrt_i_swap
    assert_eq!(
        Op::sqrt_i_swap(0b11).matrix_t::<4>(),
        [   [_1, _0,            _0,             _0],
            [_0, SQRT_1_2 * _1, SQRT_1_2 * _i,  _0],
            [_0, SQRT_1_2 * _i, SQRT_1_2 * _1,  _0],
            [_0, _0,            _0,             _1]]
    );
}

#[test]
fn rotate_operators() {
    const PHASE: R = 1.1;

    println!("{:?}", (PHASE * 0.5).sin_cos());

    //rx
    println!("{:?}", get_matrix_2x2(&Op::rx(PHASE, 0b1)));
    //ry
    println!("{:?}", get_matrix_2x2(&Op::ry(PHASE, 0b1)));
    //rz
    println!("{:?}", get_matrix_2x2(&Op::rz(PHASE, 0b1)));

    //rxx
    println!("{:?}", get_matrix_4x4(&Op::rxx(PHASE, 0b11)));
    //ryy
    println!("{:?}", get_matrix_4x4(&Op::ryy(PHASE, 0b11)));
    //rzz
    println!("{:?}", get_matrix_4x4(&Op::rzz(PHASE, 0b11)));
}