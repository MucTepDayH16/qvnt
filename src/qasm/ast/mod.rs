use qasm::{self, AstNode};

mod error;
pub use error::*;

#[derive(Clone, Debug, PartialEq,)]
pub struct Ast<'t,> {
    source: &'t str,
    ast: Vec<AstNode<'t,>,>,
}

impl<'t,> Ast<'t,> {
    pub fn from_source(source: &'t str,) -> Result<'t, Self,> {
        let processed = qasm::pre_process(source,);
        let token_tree = qasm::lex(processed,);
        if token_tree.is_empty() {
            Err(Error::EmptySource,)
        } else {
            match qasm::parse(token_tree,) {
                Ok(ast,) => Ok(Self { source, ast, },),
                Err(err,) => Err(Error::ParseError(err,),),
            }
        }
    }

    pub fn source(&self,) -> &'t str {
        self.source
    }

    pub(crate) fn iter(&self,) -> impl Iterator<Item = &AstNode<'t,>,> {
        self.ast.iter()
    }
}

impl<'t,> IntoIterator for Ast<'t,> {
    type Item = AstNode<'t,>;
    type IntoIter = std::vec::IntoIter<Self::Item,>;

    fn into_iter(self,) -> Self::IntoIter {
        self.ast.into_iter()
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
                source: "OPENQASM 2.0; qreg a[3]; rx(pi, sqrt(2.0)) a[0], a[1];",
                ast: vec![
                    QReg("a", 3),
                    ApplyGate(
                        "rx",
                        vec![Argument::Qubit("a", 0), Argument::Qubit("a", 1),],
                        vec!["pi", "sqrt(2.0)",]
                    ),
                ]
            }),
        );

        assert_eq!(Ast::from_source(""), Err(Error::EmptySource),);
        assert_eq!(
            Ast::from_source("qreg a[3];"),
            Ok(Ast {
                source: "qreg a[3];",
                ast: vec![QReg("a", 3)]
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

        let source = std::fs::read_to_string("./src/qasm/examples/test.qasm",).unwrap();

        assert_eq!(
            Ast::from_source(&source[..]),
            Ok(Ast {
                source: include_str!("../examples/test.qasm"),
                ast: vec![
                    QReg("q", 2),
                    CReg("c", 2),
                    Gate(
                        "foo",
                        vec!["a", "b"],
                        vec!["x", "y"],
                        vec![ApplyGate("rx", vec![Register("a")], vec!["x"])]
                    ),
                    ApplyGate("h", vec![Qubit("q", 0)], vec![]),
                    ApplyGate("cx", vec![Qubit("q", 0), Qubit("q", 1)], vec![]),
                    ApplyGate(
                        "foo",
                        vec![Qubit("q", 0), Qubit("q", 1)],
                        vec!["3.141592653589793", "0"]
                    )
                ]
            }),
        );
    }
}
