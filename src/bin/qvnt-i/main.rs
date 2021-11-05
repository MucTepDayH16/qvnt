use {
    std::{
        collections::HashMap,
        path::PathBuf,
    },
    clap::{Arg, App},
    rustyline::{error::ReadlineError, Editor},
    qvnt::prelude::*,
};

include!{ "loop_fn.rs" }
include!{ "int_tree.rs" }

const VERSION: &str = "0.3.2";
const SIGN: &str = "|Q> ";
const BLCK: &str = "... ";

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
    let mut int_set = IntSet::with_root("'");
	
	let dbg = cli.is_present("debug");
	
	fn process(int: &mut Int, int_set: &mut IntSet, line: String, dbg: bool) {
		if let Err(err) = loop_fn(int, int_set, line) {
			if dbg {
				eprintln!("{:?}\n", err);
			} else {
				eprintln!("{}\n", err);
			}
		}
	}

    print!("{}", PROLOGUE);
    let mut interact = Editor::<()>::new();
    let _ = interact.load_history(".history");
	let mut block = (false, String::new());
	
    loop {
        match interact.readline(if block.0 {BLCK} else {SIGN}) {
            Ok(line) => {
				interact.add_history_entry(&line);
				match line.chars().last() {
					Some('{') => {
						block.1 += &line;
						block.0 = true;
					},
					Some('}') if block.0 => {
						block.1 += &line;
						block.0 = false;
						process(&mut int, &mut int_set, block.1, dbg);
                        block.1 = String::new();
					},
					_ if block.0 => {
						block.1 += &line;
					},
					_ => {
						process(&mut int, &mut int_set, line, dbg);
					},
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

    let _ = interact.save_history(".history");
    println!("{}", EPILOGUE);
}