#[cfg(test)]
mod tests;

mod types;
mod math;
mod bits_iter;

pub mod prelude {
    pub use super::operator::Op;
    pub use super::register::{VReg, QReg};
    pub use super::threads::qvnt_num_threads;
}

pub mod operator;
pub mod register;
pub mod threads;