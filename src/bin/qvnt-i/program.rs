use crate::{
    cli::CliArgs,
    int_tree::IntTree,
    process::{Error, Process, Result},
};
use qvnt::prelude::Int;
use rustyline::{error::ReadlineError, Editor};

pub fn leak_string<'t>(s: String) -> &'t str {
    let s = Box::leak(s.into_boxed_str()) as &'t str;
    // eprintln!("Leakage {{ ptr: {:?}, len: {} }}", s as *const _, s.len());
    s
}

pub(crate) struct Program<'t> {
    pub dbg: bool,
    pub interact: Editor<()>,
    pub curr_process: Process<'t>,
    pub int_tree: IntTree<'t>,
}

fn handle_error(result: Result, dbg: bool) -> Option<i32> {
    match result {
        Ok(()) => None,
        Err(Error::Inner | Error::Unimplemented) => {
            eprintln!("Internal Error: Please report this to the developer.");
            Some(0xDE)
        }
        Err(Error::Dyn(err)) => {
            if dbg {
                eprintln!("{:?}\n", err);
            } else {
                eprintln!("{}\n", err);
            }
            None
        }
        Err(Error::Quit(n)) => Some(n),
    }
}

impl<'t> Program<'t> {
    pub fn new(cli: CliArgs) -> std::result::Result<Self, ()> {
        const PROLOGUE: &str = "QVNT - Interactive QASM Interpreter\n\n";
        print!("{}", PROLOGUE);

        let mut interact = Editor::new();
        let _ = interact.load_history(&cli.history);

        let mut new = Self {
            dbg: cli.dbg,
            interact,
            curr_process: Process::new(Int::default()),
            int_tree: IntTree::with_root(""),
        };

        if let Some(path) = cli.input {
            if let Some(n) = handle_error(
                new.curr_process.load_qasm(&mut new.int_tree, path.into()),
                new.dbg,
            ) {
                if n != 0 {
                    return Err(());
                }
            }
        }

        Ok(new)
    }

    pub fn run(mut self) -> std::result::Result<(), ()> {
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
                            let line = leak_string(std::mem::take(&mut block.1));
                            if let Some(n) =
                                handle_error(self.curr_process.process_qasm(line), self.dbg)
                            {
                                if n == 0 {
                                    break Ok(());
                                } else {
                                    break Err(());
                                }
                            }
                        }
                        _ if block.0 => {
                            block.1 += &line;
                        }
                        _ => {
                            if let Some(n) = handle_error(
                                self.curr_process.process(&mut self.int_tree, line),
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

    use crate::{int_tree::IntTree, process::*, program::leak_string};

    #[test]
    fn main_loop() {
        let mut int_tree = IntTree::with_root("");
        let mut curr_process = Process::new(Int::default());
        let mut block = (false, String::new());

        let input = vec![
            (":tags", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: [], macros: {}, .. }"),
            ("qreg q[4];", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: [], macros: {}, .. }"),
            (":tag reg", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: [], macros: {}, .. }"),
            ("h q[2];", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: [H4], macros: {}, .. }"),
            (":tag ops", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: [H4], macros: {}, .. }"),
            (":to reg", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: [], macros: {}, .. }"),
            (":reset", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: [], macros: {}, .. }"),
            ("gate OOO(a, b) x, y { h x; rx(a+b) y; }", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: [], macros: {\"OOO\": Macro { regs: [\"x\", \"y\"], args: [\"a\", \"b\"], nodes: [(\"h\", [Register(\"x\")], []), (\"rx\", [Register(\"y\")], [\"a+b\"])] }}, .. }"),
            (":tag macro", "Int { m_op: Set, q_reg: [], c_reg: [], q_ops: [], macros: {\"OOO\": Macro { regs: [\"x\", \"y\"], args: [\"a\", \"b\"], nodes: [(\"h\", [Register(\"x\")], []), (\"rx\", [Register(\"y\")], [\"a+b\"])] }}, .. }"),
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
                    let line = leak_string(block.1);
                    curr_process.process_qasm(line).unwrap();
                    block.1 = String::new();
                }
                _ if block.0 => {
                    block.1 += &line;
                }
                _ => {
                    curr_process.process(&mut int_tree, line).unwrap();
                }
            }

            assert_eq!(
                format!("{:?}", curr_process.int()),
                expected_int.to_string()
            );
        }
    }
}
