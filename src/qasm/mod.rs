pub(crate) mod ast;
pub(crate) mod int;
pub(crate) mod sym;

pub use ast::Ast;
pub use int::Int;
pub use sym::Sym;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::CReg;

    #[test]
    fn circuit() {
        let ast = Ast::from_file(&"./src/qasm/examples/test.qasm".to_string()).unwrap();
        let int = Int::new(&ast).unwrap();
        let mut sym = Sym::new(int);

        let mut hist = vec![0; 4];

        for _ in 0..1000 {
            sym.reset();
            sym.finish();
            hist[sym.get_class().get_by_mask(0b11)] += 1;
        }

        println!("{:?}", hist);
    }
}
