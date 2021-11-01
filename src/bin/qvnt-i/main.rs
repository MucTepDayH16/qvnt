#![cfg(feature = "interpreter")]

use {
    std::path::PathBuf,
    clap::{Arg, App, SubCommand},
    rustyline::{error::ReadlineError, Editor},
    qvnt::prelude::*,
};

const VERSION: &str = "0.3.2";
const SIGN: &str = "|Q> ";

const PROLOGUE: &str = "QVNT - Interactive QASM Interpreter\n\n";
const EPILOGUE: &str = "";

fn loop_fn(int: &mut Int, int_stack: &mut Vec<Int>, line: &String) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    match line.chars().next() {
        Some(':') => {
            let mut line = line[1..].split_whitespace();
            while let Some(cmd) = line.next() {
                match cmd {
                    "push" =>
                        int_stack.push(int.clone()),
                    "pop" =>
                        match int_stack.pop() {
                            Some(popped) => *int = popped,
                            None => eprintln!("nothing to pop\n"),
                        }
                    "exit" | "quit" | "q" =>
                        std::process::exit(0),
                    "class" =>
                        println!("{}\n", int.get_class().get_value(!0usize)),
                    "polar" =>
                        println!("{:.2?}\n", int.get_polar_wavefunction()),
                    "prob" =>
                        println!("{:.2?}\n", int.get_probabilities()),
                    "ops" =>
                        println!("{}\n", int.get_ops_tree()),
                    "finish" | "f" => {
                        int.reset().finish();
                    },
                    "reset" | "r" => {
                        *int = Int::default();
                    }
                    "alias" | "a" | "names" => {
                        println!("QREGs {}\nCREGs {}\n", int.get_q_alias(), int.get_c_alias())
                    }
                    "load" | "l" => {
                        match line.next() {
                            Some(path) => {
                                let path = std::path::PathBuf::from(path);
                                let ast = Ast::from_file(&path)?;
                                std::mem::drop(std::mem::take(int));
                                *int = Int::new(&ast)?;
                            },
                            None => eprintln!("you must specify path to load!\n"),
                        }
                    }
                    _ => eprintln!("unknown command!\n"),
                }
            }
        },
        _ => {
            let line = "OPENQASM 2.0; ".to_string() + &line;
            let ast = Ast::from_source(line)?;
            int.add(&ast)?;
            //  int.reset().finish();
        },
    }

    Ok(())
}

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

    print!("{}", PROLOGUE);
    let mut interact = Editor::<()>::new();
    let _ = interact.load_history(".history");
    loop {
        match interact.readline(SIGN) {
            Ok(line) => {
                interact.add_history_entry(&line);
                if let Err(err) = loop_fn(&mut int, &mut int_stack, &line) {
                    eprintln!("{}\n", err);
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