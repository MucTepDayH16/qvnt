use std::error::Error;
use std::io::Write;
use {
    std::{
        io::{BufRead, Read, stdin, stdout},
        path::PathBuf,
    },
    clap::{Arg, App, SubCommand},
    qvnt::prelude::*,
};

const VERSION: &str = "0.3.2";
const SIGN: &str = "|i>\t";

fn loop_fn(mut int: Int, buf: &String) -> Result<Int, String> {
    match buf.chars().next() {
        Some(':') => {
            let mut buf = buf.chars();
            buf.next();
            let buf = buf
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();
            match &buf[..] {
                "class" => println!("{}", int.get_class().get_value(!0usize)),
                "polar" => println!("{:?}", int.get_polar_wavefunction()),
                "prob" => println!("{:?}", int.get_probabilities()),
                "quit" | "q" => std::process::exit(0),
                "finish" | "f" => int.reset().finish(),
                _ => println!("Unknown command!"),
            }
        },
        _ => {
            let buf = "OPENQASM 2.0; ".to_string() + &buf;
            let ast = Ast::from_source(buf)
                .map_err(|err| err.to_string())?;
            int = int.add(&ast)
                .map_err(|err| err.to_string())?;
            int.reset().finish();
        },
    }

    Ok(int)
}

fn main() {
    let cli = App::new("QVNT Interpreter").version(VERSION).author("Denis Drozhzhin <denisdrozhzhin1999@gmail.com>").arg(
        Arg::with_name("input").short("i").long("input").value_name("FILE").help("Specify QASM file path").takes_value(true)
    ).get_matches();

    let mut int = match cli.value_of("input") {
        Some(input) => {
            let path = input.parse::<PathBuf>().unwrap();
            let ast = Ast::from_file(&path).unwrap();
            Int::new(&ast).unwrap()
        }
        None => Int::default(),
    };

    let mut history = vec![];

    loop {
        let mut buf = String::new();
        stdin().lock().read_line(&mut buf).unwrap();

        match loop_fn(int.clone(), &buf) {
            Ok(next) => int = next,
            Err(err) => eprintln!("{}", err),
        }

        history.push(buf.clone());
    }
}