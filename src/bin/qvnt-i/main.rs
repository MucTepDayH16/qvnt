use {
    std::path::PathBuf,
    clap::{Arg, App, SubCommand},
    qvnt::prelude::*,
};

const VERSION: &str = "0.3.2";
const SIGN: &str = "|i>\t";

macro_rules! scanln {
    () => {{
        use std::io::{BufRead, Read, stdin};

        let mut buf = String::new();
        stdin().lock().read_line(&mut buf).map(|_| buf)
    }};
}

fn loop_fn(int: &mut Int, buf: &String) -> Result<(), Box<dyn std::error::Error>> {
    match buf.chars().next() {
        Some(':') => {
            let buf = &buf[1..]
                .split_whitespace()
                .collect::<Vec<&str>>();
            let len = buf.len();

            match buf.get(0) {
                Some(&"class") =>
                    println!("{}", int.get_class().get_value(!0usize)),
                Some(&"polar") =>
                    println!("{:?}", int.get_polar_wavefunction()),
                Some(&"prob") =>
                    println!("{:?}", int.get_probabilities()),
                Some(&"exit" | &"quit" | &"q") =>
                    std::process::exit(
                        buf.get(1)
                            .and_then(|s| s.parse::<i32>().ok())
                            .unwrap_or(0)
                    ),
                Some(&"finish" | &"f") => {
                    int.reset().finish();
                },
                _ => println!("Unknown command!"),
            }
        },
        _ => {
            let buf = "OPENQASM 2.0; ".to_string() + &buf;
            let ast = Ast::from_source(buf)
                .map_err(|err| err.to_string())?;
            int.add(&ast)
                .map_err(|err| err.to_string())?;
            //  int.reset().finish();
        },
    }

    Ok(())
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
        let cmd = scanln!().expect("read line error");
        if let Err(err) = loop_fn(&mut int, &cmd) {
            eprintln!("{}", err);
        }

        history.push(cmd);
    }
}