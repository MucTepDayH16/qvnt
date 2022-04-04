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
