pub (crate) mod ast;
pub (crate) mod int;

pub use ast::Ast;
pub use int::Int;

#[cfg(test)]
mod tests {
    use crate::register::CReg;
    use super::*;

    #[test]
    fn circuit() {
        let ast = Ast::from_file(&"./src/qasm/examples/test.qasm".to_string()).unwrap();
        let mut int = Int::new(&ast).unwrap();

        for _ in 0..100 {
            int.reset().finish();
            assert!(
                int.get_class() == CReg::new(2).init_state(0) ||
                    int.get_class() == CReg::new(2).init_state(3));
        }
    }
}