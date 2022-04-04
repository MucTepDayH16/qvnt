use std::path::PathBuf;

use crate::{
    cli::CliArgs,
    int_tree::IntTree,
    process::{handle_error, process, process_qasm, Process},
};
use qvnt::prelude::{Ast, Int};
use rustyline::{error::ReadlineError, Editor};

pub(crate) struct Program {
    pub dbg: bool,
    pub interact: Editor<()>,
    pub curr_process: Process,
    pub int_tree: IntTree,
}

impl Program {
    pub fn new(cli: CliArgs) -> Self {
        const PROLOGUE: &str = "QVNT - Interactive QASM Interpreter\n\n";
        print!("{}", PROLOGUE);

        let mut int_tree = IntTree::with_root("");

        let maybe_int = |input| -> Result<_, Box<dyn std::error::Error>> {
            match input {
                Some(input) => {
                    let path = PathBuf::from(input);
                    let ast = Ast::from_file(&path)?;

                    let path_tag = crate::process::file_tag(&path);
                    Ok((Int::new(&ast)?, Some(path_tag)))
                }
                None => Ok((Int::default(), None)),
            }
        };

        let int = match maybe_int(cli.input) {
            Ok((int, maybe_tag)) => {
                if let Some(tag) = maybe_tag {
                    int_tree.commit(tag, int.clone());
                }
                int
            }
            Err(err) => {
                if cli.dbg {
                    eprintln!("{:?}\n", err);
                } else {
                    eprintln!("{}\n", err);
                }
                Int::default()
            }
        };

        let mut interact = Editor::new();
        let _ = interact.load_history(&cli.history);

        Self {
            dbg: cli.dbg,
            interact,
            curr_process: Process::new(int),
            int_tree,
        }
    }

    pub fn run(mut self) -> Result<(), ()> {
        const SIGN: &str = "|Q> ";
        const BLCK: &str = "... ";

        let mut block = (false, String::new());
        let ret_code = loop {
            match self.interact.readline(if block.0 { BLCK } else { SIGN }) {
                Ok(line) => {
                    println!();
                    self.interact.add_history_entry(&line);
                    match line.chars().last() {
                        Some('{') => {
                            block.1 += &line;
                            block.0 = true;
                        }
                        Some('}') if block.0 => {
                            block.1 += &line;
                            block.0 = false;
                            if let Some(n) = handle_error(
                                process_qasm(&mut self.curr_process, block.1),
                                self.dbg,
                            ) {
                                if n == 0 {
                                    break Ok(());
                                } else {
                                    break Err(());
                                }
                            }
                            block.1 = String::new();
                        }
                        _ if block.0 => {
                            block.1 += &line;
                        }
                        _ => {
                            if let Some(n) = handle_error(
                                process(&mut self.curr_process, &mut self.int_tree, line),
                                self.dbg,
                            ) {
                                if n == 0 {
                                    break Ok(());
                                } else {
                                    break Err(());
                                }
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    eprintln!("\nExit: Keyboard Interrupted");
                    break Ok(());
                }
                Err(ReadlineError::Eof) => {
                    eprintln!("\nExit: End of File");
                    break Err(());
                }
                Err(err) => {
                    eprintln!("\nError: {:?}", err);
                    break Err(());
                }
            }
        };

        let _ = self.interact.save_history(".history");
        ret_code
    }
}

#[cfg(test)]
mod tests {
    use qvnt::prelude::Int;

    use crate::{int_tree::IntTree, process::*};

    #[test]
    fn main_loop() {
        let mut int_tree = IntTree::with_root("");
        let mut curr_process = Process::new(Int::default());
        let mut block = (false, String::new());

        let input = vec![
            (":tags", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: , macros: {} }"),
            ("qreg q[4];", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: , macros: {} }"),
            (":tag reg", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: , macros: {} }"),
            ("h q[2];", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops:  -> [H4], macros: {} }"),
            (":tag ops", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops:  -> [H4], macros: {} }"),
            (":to reg", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: , macros: {} }"),
            (":reset", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: , macros: {} }"),
            ("gate OOO(a, b) x, y { h x; rx(a+b) y; }", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: , macros: {\"OOO\": Macro { regs: [\"x\", \"y\"], args: [\"a\", \"b\"], nodes: [(\"h\", [Register(\"x\")], []), (\"rx\", [Register(\"y\")], [\"a+b\"])] }} }"),
            (":tag macro", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: , macros: {\"OOO\": Macro { regs: [\"x\", \"y\"], args: [\"a\", \"b\"], nodes: [(\"h\", [Register(\"x\")], []), (\"rx\", [Register(\"y\")], [\"a+b\"])] }} }"),
        ];

        for (line, expected_int) in input {
            let line = line.to_string();
            match line.chars().last() {
                Some('{') => {
                    block.1 += &line;
                    block.0 = true;
                }
                Some('}') if block.0 => {
                    block.1 += &line;
                    block.0 = false;
                    process_qasm(&mut curr_process, block.1).unwrap();
                    block.1 = String::new();
                }
                _ if block.0 => {
                    block.1 += &line;
                }
                _ => {
                    process(&mut curr_process, &mut int_tree, line).unwrap();
                }
            }

            assert_eq!(
                format!("{:?}", curr_process.int()),
                expected_int.to_string()
            );
        }
    }
}
