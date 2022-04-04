use {
    clap::{App, Arg},
    int_tree::IntTree,
    qvnt::prelude::*,
    rustyline::{error::ReadlineError, Editor},
};

mod commands;
mod int;
mod int_tree;
mod process;

use process::*;

const VERSION: &str = "0.4.0-beta";
const SIGN: &str = "|Q> ";
const BLCK: &str = "... ";

const PROLOGUE: &str = "QVNT - Interactive QASM Interpreter\n\n";

fn main() {
    let cli = App::new("QVNT Interpreter")
        .version(VERSION)
        .author("Denis Drozhzhin <denisdrozhzhin1999@gmail.com>")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Specify QASM file path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("dbg")
                .help("Set debug format for errors"),
        )
        .get_matches();

    let dbg = cli.is_present("debug");
    let mut int_tree = IntTree::with_root("");

    let int = match int::from_cli(&cli) {
        Ok((int, maybe_tag)) => {
            if let Some(tag) = maybe_tag {
                int_tree.commit(tag, int.clone());
            }
            int
        },
        Err(err) => {
            if dbg {
                eprintln!("{:?}\n", err);
            } else {
                eprintln!("{}\n", err);
            }
            Int::default()
        }
    };

    let mut curr_process = Process::new(int);

    print!("{}", PROLOGUE);
    let mut interact = Editor::<()>::new();
    let _ = interact.load_history(".history");
    let mut block = (false, String::new());

    let code = loop {
        match interact.readline(if block.0 { BLCK } else { SIGN }) {
            Ok(line) => {
                println!();
                interact.add_history_entry(&line);
                match line.chars().last() {
                    Some('{') => {
                        block.1 += &line;
                        block.0 = true;
                    }
                    Some('}') if block.0 => {
                        block.1 += &line;
                        block.0 = false;
                        if let Some(n) = handle_error(process_qasm(&mut curr_process, block.1), dbg) {
                            break n;
                        }
                        block.1 = String::new();
                    }
                    _ if block.0 => {
                        block.1 += &line;
                    }
                    _ => {
                        if let Some(n) = handle_error(process(&mut curr_process, &mut int_tree, line), dbg) {
                            break n;
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("\nExit: Keyboard Interrupted");
                break 1;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("\nExit: End of File");
                break 2;
            }
            Err(err) => {
                eprintln!("\nError: {:?}", err);
                break 3;
            }
        }
    };

    let _ = interact.save_history(".history");

    std::process::exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            (":goto reg", "Int { m_op: Set, q_reg: [\"q\", \"q\", \"q\", \"q\"], c_reg: [], q_ops: , macros: {} }"),
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

            assert_eq!(format!("{:?}", curr_process.int()), expected_int.to_string());
        }
    }
}