//! Module contains definitions for quantum and classical registers.
//!
//! QVNT provide 3 types of registers:
//! * [`QReg`] - quantum register;
//! * [`CReg`] - classical register;
//! * [`VReg`] - *vurtual* register.
//!
//! [`QReg`] is used when you need to build and apply quantum circuit.
//!

pub (crate) mod virtl;
pub (crate) mod quant;
pub (crate) mod class;

pub use quant::Reg as QReg;
pub use class::Reg as CReg;
pub use virtl::Reg as VReg;

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn quantum_reg() {
        let mut reg = QReg::new(4)
            .init_state(0b1100);
        let mask = 0b0110;

        let mut operator =
            op::h(0b1111)
                * op::h(0b0011).c(0b1000).unwrap()
                * op::swap(0b1001).c(0b0010).unwrap()
            ;

        reg.apply(&operator);

        assert_eq!(format!("{:?}", operator), "[H3, H12, C8_H3, C2_SWAP9]");
        assert_eq!(format!("{:?}", reg), "[Complex { re: 0.25, im: 0.0 }, Complex { re: 0.25, im: 0.0 }, Complex { re: 0.25, im: 0.0 }, Complex { re: 0.0, im: 0.0 }, Complex { re: -0.25, im: 0.0 }, Complex { re: -0.25, im: 0.0 }, Complex { re: -0.25, im: 0.0 }, Complex { re: 0.0, im: 0.0 }]");

        operator.clear();
        assert_eq!(operator.len(), 0);
        assert_eq!(reg.measure_mask(mask).get() & !mask, 0);
    }

    #[test]
    fn tensor() {
        const EPS: f64 = 1e-9;

        let pend_ops = op::h(0b01);

        let mut reg1 = QReg::new(2).init_state(0b01);
        let mut reg2 = QReg::new(1).init_state(0b1);

        reg1.apply(&pend_ops);
        reg2.apply(&pend_ops);

        let test_prob = (reg1 * reg2).get_probabilities();
        let true_prob = vec![0.25, 0.25, 0., 0., 0.25, 0.25, 0., 0.];

        assert!(
            test_prob.into_iter().zip(true_prob.into_iter())
                     .all(|(a, b)| (a - b).abs() < EPS)
        );

        let mut reg3 = QReg::new(3).init_state(0b101);
        let pend_ops = op::h(0b101);

        reg3.apply(&pend_ops);

        let test_prob = reg3.get_probabilities();
        let true_prob = vec![0.25, 0.25, 0., 0., 0.25, 0.25, 0., 0.];

        assert!(
            test_prob.into_iter().zip(true_prob.into_iter())
                     .all(|(a, b)| (a - b).abs() < EPS)
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
        let m = reg.get_vreg_by(0b01101).unwrap();
        let x = reg.get_vreg_by(0b00111).unwrap();
        let y = reg.get_vreg_by(0b11000).unwrap();

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
}