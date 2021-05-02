pub use num::{
    abs,
    Complex,
    Integer,
    traits::{
        Inv,
        FloatConst,
        real::Real,
    },
    One,
    Zero,
};

pub mod nums {
    pub type N = usize;
    pub type Z = isize;

    pub type R = f64;
    pub type C = super::Complex<R>;
}