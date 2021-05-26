#[cfg(test)]
mod tests;

mod types;
mod math;
mod bits_iter;

pub mod prelude {
    use super::operator::Op;
    use super::register::{QReg, VReg};
}

pub mod operator;
pub mod register;