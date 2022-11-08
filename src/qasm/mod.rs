pub mod ast;
pub mod int;
pub mod sym;

pub use ast::Ast;
pub use int::Int;
pub use sym::Sym;

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::backend::DefaultBuilder;

    #[test_case(include_str!("./examples/source/adder.qasm"); "adder")]
    #[test_case(include_str!("./examples/source/bigadder.qasm"); "big_adder")]
    #[test_case(include_str!("./examples/source/Deutsch_Algorithm.qasm"); "Deutsch_Algorithm")]
    #[test_case(include_str!("./examples/source/inverseqft1.qasm"); "inverse_qft_1")]
    #[test_case(include_str!("./examples/source/inverseqft2.qasm"); "inverse_qft_2")]
    #[test_case(include_str!("./examples/source/ipea_3_pi_8.qasm"); "ipea_3_pi_8")]
    #[test_case(include_str!("./examples/source/qe_qft_3.qasm"); "qe_qft_3")]
    #[test_case(include_str!("./examples/source/qe_qft_4.qasm"); "qe_qft_4")]
    #[test_case(include_str!("./examples/source/qe_qft_5.qasm"); "qe_qft_5")]
    #[test_case(include_str!("./examples/source/qec.qasm"); "qec")]
    #[test_case(include_str!("./examples/source/qft.qasm"); "qft")]
    #[test_case(include_str!("./examples/source/qpt.qasm"); "qpt")]
    #[test_case(include_str!("./examples/source/rb.qasm"); "rb")]
    #[test_case(include_str!("./examples/source/teleport.qasm"); "teleport")]
    #[test_case(include_str!("./examples/source/teleportv2.qasm"); "teleport_v2")]
    #[test_case(include_str!("./examples/source/W-state.qasm"); "w_state")]
    #[test_case(include_str!("./examples/source/W3test.qasm"); "W3test")]
    fn run_qasm(source: &'static str) {
        let ast = Ast::from_source(source).unwrap();
        let int = Int::new(ast).unwrap();
        let mut sym = Sym::new(int, DefaultBuilder::default());

        sym.reset();
        sym.finish();
    }
}
