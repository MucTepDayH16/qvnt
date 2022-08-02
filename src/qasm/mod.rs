pub mod ast;
pub mod int;
pub mod sym;

pub use ast::Ast;
pub use int::Int;
pub use sym::Sym;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuits() {
        for path in glob::glob("./src/qasm/examples/source/*.qasm").unwrap() {
            let path = path.unwrap();
            let file_name = path.file_name().and_then(|f| f.to_str()).unwrap();
            if file_name == "qelib1.qasm" {
                continue;
            }

            let source = std::fs::read_to_string(path).unwrap();
            let ast = Ast::from_source(&source[..]).unwrap();
            let int = Int::new(ast).unwrap();
            let mut sym = Sym::new(int);

            sym.reset();
            sym.finish();
        }
    }
}
