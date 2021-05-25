#[cfg(test)]
mod tests;

mod types;
mod math;

pub mod prelude {
    use super::operator::Op;
    use super::register::{QReg, VReg};
}

pub mod operator;
pub mod register;