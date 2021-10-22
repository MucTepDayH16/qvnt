use {
    std::{
        fs::File,
        io::Read,
        path::Path,
    },
    qasm::{AstNode, process, lex, parse},
};

mod error;
pub use error::*;

#[derive(Debug)]
pub struct Ast {
    ast: Vec<AstNode>,
}

impl Ast {
    pub fn from_source<S: ToString>(source: S) -> Result<Self> {
        let mut tokens = lex(&source.to_string());
        if tokens.is_empty() {
            Err(Error::EmptySource)
        } else {
            parse(&mut tokens)
                .map(|ast| Self{ ast })
                .map_err(|err| Error::ParseError(err))
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: &P) -> Result<Self> {
        let mut source = "".to_string();
        let mut file = File::open(path)
            .map_err(|_| Error::NoSuchFile(path.as_ref().to_path_buf()))?;
        file.read_to_string(&mut source)
            .map_err(|_| Error::CannotRead(path.as_ref().to_path_buf()))?;

        let source = process(&source, &std::env::current_dir().unwrap());
        Self::from_source(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_from_source() {
        let _ = Ast::from_source("OPENQASM 2.0; qreg a[3]; CX a[0], a[1];").unwrap();

        assert_eq!(Ast::from_source("").unwrap_err(),
                   Error::EmptySource);
        assert_eq!(Ast::from_source("qreg a[3]; CX a[0], a[1];").unwrap_err(),
                   Error::ParseError(qasm::Error::MissingVersion));
        assert_eq!(Ast::from_source("OPENQASM 0.0; qreg a[3]; CX a[0], a[1];").unwrap_err(),
                   Error::ParseError(qasm::Error::UnsupportedVersion));
        assert_eq!(Ast::from_source("OPENQASM 2.0 qreg a[3]; CX a[0], a[1];").unwrap_err(),
                   Error::ParseError(qasm::Error::MissingSemicolon));
        assert_eq!(Ast::from_source("OPENQASM 2.0; qreg a[]; CX a[0], a[1];").unwrap_err(),
                   Error::ParseError(qasm::Error::MissingInt));
        assert_eq!(Ast::from_source("OPENQASM 2.0; qreg a[3]; a[0], a[1];").unwrap_err(),
                   Error::ParseError(qasm::Error::MissingIdentifier));
    }

    #[test]
    fn ast_from_file() {
        let _ = Ast::from_file(&"./src/qasm/examples/test.qasm".to_string()).unwrap();

        let p = Path::new("./src/qasm/examples/not_test.qasm");
        assert_eq!(Ast::from_file(&p).unwrap_err(),
                   Error::NoSuchFile(p.to_path_buf()));
    }
}