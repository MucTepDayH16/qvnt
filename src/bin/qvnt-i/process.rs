use crate::{IntSet, commands::{Commands, Command, self}};
use qvnt::qasm::{Ast, Int};

pub enum Error {
    Dyn(Box<dyn std::error::Error>),
    Quit(i32),
}

impl<E: std::error::Error + 'static> From<E> for Error {
    fn from(e: E) -> Self {
        Self::Dyn(e.into())
    }
}

type Result = std::result::Result<(), Error>;

pub fn handle_error(
    result: Result,
    dbg: bool
) -> Option<i32> {
    match result {
        Ok(()) => None,
        Err(Error::Dyn(err)) => {
            if dbg {
                eprintln!("{:?}\n", err);
            } else {
                eprintln!("{}\n", err);
            }
            None
        },
        Err(Error::Quit(n)) => Some(n)
    }
}

pub fn process<'a>(
    int: &mut Int,
    int_set: &mut IntSet,
    line: String,
) -> Result {
    match line.parse::<Commands>() {
        Ok(cmds) => process_cmd(int, int_set, cmds.0.into_iter()),
        Err(commands::Error::MissingCollon) => process_qasm(int, line),
        Err(err) => Err(err.into()),
    }
}

pub fn process_qasm(
    int: &mut Int,
    line: String
) -> Result {
    let line = "OPENQASM 2.0; ".to_string() + &line;
    let ast = Ast::from_source(line)?;
    int.add(&ast)?;
    Ok(())
}

pub fn process_cmd(
    int: &mut Int,
    int_set: &mut IntSet,
    mut cmds: impl Iterator<Item=Command> + Clone
) -> Result {
    while let Some(cmd) = cmds.next() {
        match cmd {
            Command::Loop(n) => {
                for _ in 0..n {
                    process_cmd(int, int_set, cmds.clone())?;
                }
                break;
            },
            Command::Tags => {
                println!(
                    "{:?}\n",
                    int_set.map.keys().collect::<Vec<&(String, String)>>(),
                );
            },
            Command::Tag(tag) => {
                if let Some(_) = int_set.tag(&tag[..], int) {
                    println!("Replace tag {:?}\n", tag);
                }
            },
            Command::Goto(tag) => {
                if let Some(to_replace) = int_set.goto(&tag[..]) {
                    *int = to_replace.clone();
                    println!("Goto tag {:?}\n", tag);
                } else {
                    return Err(commands::Error::WrongTagName(tag).into());
                }
            },
            Command::Go => {
                int.reset().finish();
            },
            Command::Reset => {
                *int = Int::default();
            },
            Command::Load(path) => {
                let ast = Ast::from_file(&path)?;
                *int = Int::new(&ast)?;
            },
            Command::Class => {
                println!("CReg: {}\n", int.get_class().get())
            },
            Command::Polar => {
                println!("QReg polar: {:.4?}\n", int.get_polar_wavefunction());
            },
            Command::Probs => {
                println!("QReg probabilities: {:.4?}\n", int.get_probabilities());
            },
            Command::Ops => {
                println!("Operations: {}\n", int.get_ops_tree());
            },
            Command::Names => {
                println!("QReg: {}\nCReg: {}\n", int.get_q_alias(), int.get_c_alias());
            },
            Command::Help => {
                println!("{}", commands::HELP);
            },
            Command::Quit => {
                return Err(Error::Quit(0));
            },
        }
    }

    Ok(())
    /*
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

    Ok(())*/
}