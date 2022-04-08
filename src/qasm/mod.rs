pub(crate) mod ast;
pub(crate) mod int;
pub(crate) mod sym;

pub use ast::Ast;
pub use int::Int;
pub use sym::Sym;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuits() {
        for source in glob::glob("./qasm-rust/tests/source/*.qasm").unwrap() {
            let source = source.unwrap();
            let file_name = source.file_name().and_then(|f| f.to_str()).unwrap();
            if file_name == "qelib1.qasm" {
                continue;
            }

            let ast = Ast::from_file(&source, None).unwrap();
            let int = Int::new(&ast).unwrap();
            let mut sym = Sym::new(int);

            sym.reset();
            sym.finish();
        }
    }
}
