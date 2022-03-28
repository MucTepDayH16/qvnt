use crate::IntSet;
use qvnt::qasm::{Ast, Int};

pub fn loop_fn<'a>(
    int: &mut Int,
    int_set: &mut IntSet,
    line: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    match line.chars().next() {
        Some(':') => {
            let line = line[1..].split_whitespace();
            cmd::process(int, int_set, line)?;
        }
        _ => {
            let line = "OPENQASM 2.0; ".to_string() + &line;
            let ast = Ast::from_source(line)?;
            int.add(&ast)?;
        }
    }

    Ok(())
}

mod cmd {
    use super::*;
    use std::fmt;

    #[derive(Debug, Clone, PartialEq)]
    enum Error {
        UnknownCommand(String),
        UnspecifiedPath,
        UnspecifiedInt,
        UnspecifiedTag,
        WrongTagName(String),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::UnknownCommand(s) => write!(f, "unknown command: {}", s),
                Error::UnspecifiedPath => write!(f, "you must specify path to load file"),
                Error::UnspecifiedInt => {
                    write!(f, "you must specify an integer to loop over comands")
                }
                Error::UnspecifiedTag => write!(f, "you must specify a tag name as string"),
                Error::WrongTagName(s) => write!(f, "there's no tag named \"{}\"", s),
            }
        }
    }

    impl std::error::Error for Error {}

    pub fn process<'a, I>(
        int: &mut Int,
        int_set: &mut IntSet,
        mut line: I,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        I: Iterator<Item = &'a str> + Clone,
    {
        while let Some(cmd) = line.next() {
            match cmd {
                "loop" => match line.next().and_then(|s| s.parse::<usize>().ok()) {
                    Some(num) => {
                        for _ in 0..num {
                            process(int, int_set, line.clone())?;
                        }
                    }
                    None => Err(Error::UnspecifiedInt)?,
                },
                "tags" => println!(
                    "{:?}\n",
                    int_set.map.keys().collect::<Vec<&(String, String)>>()
                ),
                "tag" => match line.next() {
                    Some(tag) => {
                        if let Some(_) = int_set.tag(tag, int.clone()) {
                            println!("replace tag \"{}\"\n", tag);
                        }
                    }
                    None => Err(Error::UnspecifiedTag)?,
                },
                "goto" => match line.next() {
                    Some(tag) => {
                        if let Some(to_replace) = int_set.goto(tag) {
                            *int = to_replace.clone();
                            println!("goto tag \"{}\"\n", tag);
                        } else {
                            Err(Error::WrongTagName(tag.to_string()))?;
                        }
                    }
                    None => Err(Error::UnspecifiedTag)?,
                },
                "exit" | "quit" | "q" => std::process::exit(0),
                "class" => println!("{}\n", int.get_class().get()),
                "polar" => println!("{:.4?}\n", int.get_polar_wavefunction()),
                "prob" => println!("{:.4?}\n", int.get_probabilities()),
                "ops" => println!("{}\n", int.get_ops_tree()),
                "finish" | "f" => {
                    int.reset().finish();
                }
                "reset" | "r" => {
                    *int = Int::default();
                }
                "alias" | "a" | "names" => {
                    println!("QREGs {}\nCREGs {}\n", int.get_q_alias(), int.get_c_alias())
                }
                "load" | "l" => match line.next() {
                    Some(path) => {
                        let path = std::path::PathBuf::from(path);
                        let ast = Ast::from_file(&path)?;
                        *int = Int::new(&ast)?;
                    }
                    None => Err(Error::UnspecifiedPath)?,
                },
                _ => Err(Error::UnknownCommand(cmd.to_string()))?,
            }
        }

        Ok(())
    }
}
