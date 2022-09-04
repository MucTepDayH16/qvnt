pub mod ast;
pub mod int;
pub mod sym;

pub use ast::Ast;
pub use int::Int;
pub use sym::Sym;

#[cfg(test)]
mod qasm_sources {
    use super::*;

    fn run_qasm<'s>(source: &'s str) {
        let ast = Ast::<'s>::from_source(source).unwrap();
        let int = Int::new(ast).unwrap();
        let mut sym = Sym::new(int);

        sym.reset();
        sym.finish();
    }

    #[test]
    fn adder() {
        let source = include_str!("./examples/source/adder.qasm");
        run_qasm(source);
    }

    #[test]
    fn bigadder() {
        let source = include_str!("./examples/source/bigadder.qasm");
        run_qasm(source);
    }

    #[test]
    fn deutsch_algorithm() {
        let source = include_str!("./examples/source/Deutsch_Algorithm.qasm");
        run_qasm(source);
    }

    #[test]
    fn inverse_qft_1() {
        let source = include_str!("./examples/source/inverseqft1.qasm");
        run_qasm(source);
    }

    #[test]
    fn inverse_qft_2() {
        let source = include_str!("./examples/source/inverseqft2.qasm");
        run_qasm(source);
    }

    #[test]
    fn ipea_3_pi_8() {
        let source = include_str!("./examples/source/ipea_3_pi_8.qasm");
        run_qasm(source);
    }

    #[test]
    fn qe_qft_3() {
        let source = include_str!("./examples/source/qe_qft_3.qasm");
        run_qasm(source);
    }

    #[test]
    fn qe_qft_4() {
        let source = include_str!("./examples/source/qe_qft_4.qasm");
        run_qasm(source);
    }

    #[test]
    fn qe_qft_5() {
        let source = include_str!("./examples/source/qe_qft_5.qasm");
        run_qasm(source);
    }

    #[test]
    fn qec() {
        let source = include_str!("./examples/source/qec.qasm");
        run_qasm(source);
    }

    #[test]
    fn qft() {
        let source = include_str!("./examples/source/qft.qasm");
        run_qasm(source);
    }

    #[test]
    fn qpt() {
        let source = include_str!("./examples/source/qpt.qasm");
        run_qasm(source);
    }

    #[test]
    fn rb() {
        let source = include_str!("./examples/source/rb.qasm");
        run_qasm(source);
    }

    #[test]
    fn teleport() {
        let source = include_str!("./examples/source/teleport.qasm");
        run_qasm(source);
    }

    #[test]
    fn teleport_v2() {
        let source = include_str!("./examples/source/teleportv2.qasm");
        run_qasm(source);
    }

    #[test]
    fn w_state() {
        let source = include_str!("./examples/source/W-state.qasm");
        run_qasm(source);
    }

    #[test]
    fn w3_test() {
        let source = include_str!("./examples/source/W3test.qasm");
        run_qasm(source);
    }
}
