pub(crate) mod ast;
pub(crate) mod int;

pub use ast::Ast;
pub use int::Int;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::CReg;

    #[test]
    fn circuit() {
        let ast = Ast::from_file(&"./src/qasm/examples/test.qasm".to_string()).unwrap();
        let mut int = Int::new(&ast).unwrap();

        let mut hist = vec![0; 4];

        for _ in 0..1000 {
            int.reset().finish();
            hist[int.get_class().get_value(0b11)] += 1;
        }

        println!("{:?}", hist);
    }
}
