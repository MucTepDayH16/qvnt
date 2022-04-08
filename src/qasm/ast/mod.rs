use std::path::PathBuf;
use {
    qasm::{self, AstNode},
    std::{fs::File, io::Read, path::Path},
};

mod error;
pub use error::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Ast {
    ast: Vec<AstNode>,
}

impl Ast {
    pub fn from_source<S: AsRef<str>>(source: S) -> Result<Self> {
        let source = qasm::process(source.as_ref(), Vec::<PathBuf>::new()).unwrap();

        let mut tokens = qasm::lex(&source);
        if tokens.is_empty() {
            Err(Error::EmptySource)
        } else {
            match qasm::parse(&mut tokens) {
                Ok(ast) => Ok(Self { ast }),
                Err(err) => Err(Error::ParseError(err)),
            }
        }
    }

    pub fn from_file<P, Cwds>(path: &P, includes: Cwds) -> Result<Self>
    where
        P: AsRef<Path>,
        Cwds: IntoIterator<Item = PathBuf>,
    {
        let mut source = "".to_string();
        let mut file =
            File::open(path).map_err(|_| Error::NoSuchFile(path.as_ref().to_path_buf()))?;
        file.read_to_string(&mut source)
            .map_err(|_| Error::CannotRead(path.as_ref().to_path_buf()))?;

        let mut includes = includes.into_iter().collect::<Vec<_>>();
        if let Some(parent) = path.as_ref().parent() {
            includes.push(parent.to_path_buf());
        }
        let source = qasm::process(&source, includes)
            .map_err(|filename| Error::NoSuchFile(PathBuf::from(filename)))?;
        Self::from_source(source)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &AstNode> {
        self.ast.iter()
    }
}

#[cfg(test)]
mod tests {
    use qasm::Argument;

    use super::*;

    #[test]
    fn ast_from_source() {
        use AstNode::*;

        assert_eq!(
            Ast::from_source("OPENQASM 2.0; qreg a[3]; rx(pi, sqrt(2.0)) a[0], a[1];"),
            Ok(Ast {
                ast: vec![
                    QReg("a".to_string(), 3),
                    ApplyGate(
                        "rx".to_string(),
                        vec![
                            Argument::Qubit("a".to_string(), 0),
                            Argument::Qubit("a".to_string(), 1),
                        ],
                        vec!["pi".to_string(), "sqrt(2)".to_string(),]
                    ),
                ]
            }),
        );

        assert_eq!(Ast::from_source(""), Err(Error::EmptySource),);
        assert_eq!(
            Ast::from_source("qreg a[3];"),
            Ok(Ast {
                ast: vec![QReg("a".to_string(), 3)]
            }),
        );
        assert_eq!(
            Ast::from_source("OPENQASM 0.0; qreg a[3]; CX a[0], a[1];"),
            Err(Error::ParseError(qasm::Error::UnsupportedVersion)),
        );
        assert_eq!(
            Ast::from_source("OPENQASM 2.0 qreg a[3]; CX a[0], a[1];"),
            Err(Error::ParseError(qasm::Error::MissingSemicolon)),
        );
        assert_eq!(
            Ast::from_source("OPENQASM 2.0; qreg a[]; CX a[0], a[1];"),
            Err(Error::ParseError(qasm::Error::MissingInt)),
        );
        assert_eq!(
            Ast::from_source("OPENQASM 2.0; qreg a[3]; a[0], a[1];"),
            Err(Error::ParseError(qasm::Error::MissingIdentifier)),
        );
    }

    #[test]
    fn ast_from_file() {
        use qasm::Argument::*;
        use AstNode::*;

        assert_eq!(
            Ast::from_file(&"./src/qasm/examples/test.qasm".to_string(), None),
            Ok(Ast {
                ast: vec![
                    QReg("q".to_string(), 2),
                    CReg("c".to_string(), 2),
                    Gate(
                        "foo".to_string(),
                        vec!["a".to_string(), "b".to_string()],
                        vec!["x".to_string(), "y".to_string()],
                        vec![ApplyGate(
                            "rx".to_string(),
                            vec![Register("a".to_string())],
                            vec!["x".to_string()]
                        )]
                    ),
                    ApplyGate("h".to_string(), vec![Qubit("q".to_string(), 0)], vec![]),
                    ApplyGate(
                        "cx".to_string(),
                        vec![Qubit("q".to_string(), 0), Qubit("q".to_string(), 1)],
                        vec![]
                    ),
                    ApplyGate(
                        "foo".to_string(),
                        vec![Qubit("q".to_string(), 0), Qubit("q".to_string(), 1)],
                        vec!["3.1415927".to_string(), "0".to_string()]
                    )
                ]
            }),
        );

        let p = Path::new("./src/qasm/examples/not_test.qasm");
        assert_eq!(
            Ast::from_file(&p, None).unwrap_err(),
            Error::NoSuchFile(p.to_path_buf())
        );
    }
}
