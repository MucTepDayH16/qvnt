pub use {
    num::{
        abs,
        Complex,
        Integer,
        traits::{
            Pow,
            Inv,
            real::Real,
        },
        One,
        Zero,
    },
    std::f32::consts::*,
};

pub mod nums {
    pub type N = usize;
    pub type Z = isize;

    pub type R = f32;
    pub type C = super::Complex<R>;
}