#![cfg(feature = "interpreter")]

use {
    std::path::PathBuf,
    clap::{Arg, App, SubCommand},
    rustyline::{error::ReadlineError, Editor},
    qvnt::prelude::*,
};

include!{ "loop_fn.rs" }

const VERSION: &str = "0.3.2";
const SIGN: &str = "|Q> ";

const PROLOGUE: &str = "QVNT - Interactive QASM Interpreter\n\n";
const EPILOGUE: &str = "";

fn main() {
    let cli =
        App::new("QVNT Interpreter")
            .version(VERSION)
            .author("Denis Drozhzhin <denisdrozhzhin1999@gmail.com>")
            .arg(
                Arg::with_name("input")
                    .short("i")
                    .long("input")
                    .value_name("FILE")
                    .help("Specify QASM file path")
                    .takes_value(true)
            ).arg(
				Arg::with_name("debug")
					.short("d")
					.long("dbg")
					.help("Set debug format for errors")
			).get_matches();

    let mut int = match cli.value_of("input") {
        Some(input) => {
            let path = input.parse::<PathBuf>().unwrap();
            let ast = Ast::from_file(&path).unwrap();
            Int::new(&ast).unwrap()
        }
        None => Int::default(),
    };
    let mut int_stack = vec![];
	
	let dbg = cli.is_present("debug");

    print!("{}", PROLOGUE);
    let mut interact = Editor::<()>::new();
    let _ = interact.load_history(".history");
    loop {
        match interact.readline(SIGN) {
            Ok(line) => {
                interact.add_history_entry(&line);
                if let Err(err) = loop_fn(&mut int, &mut int_stack, &line) {
					if dbg {
						eprintln!("{:?}\n", err);
					} else {
						eprintln!("{}\n", err);
					}
                }
            },
            Err(ReadlineError::Interrupted) => {
                eprintln!("Exit: Keyboard Interrupted");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("Exit: End of File");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    interact.save_history(".history");
    println!("{}", EPILOGUE);
}