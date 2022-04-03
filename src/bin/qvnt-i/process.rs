use crate::{
    commands::{self, Command, Commands},
    IntTree,
};
use qvnt::qasm::{Ast, Int, Sym};

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

pub fn handle_error(result: Result, dbg: bool) -> Option<i32> {
    match result {
        Ok(()) => None,
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

pub fn process<'a>(int_sym: &mut (Int, Sym), int_set: &mut IntTree, line: String) -> Result {
    match line.parse::<Commands>() {
        Ok(cmds) => process_cmd(int_sym, int_set, cmds.0.into_iter()),
        Err(commands::Error::MissingCollon) => process_qasm(int_sym, line),
        Err(err) => Err(err.into()),
    }
}

pub fn process_qasm(int_sym: &mut (Int, Sym), line: String) -> Result {
    let line = "OPENQASM 2.0; ".to_string() + &line;
    let ast = Ast::from_source(line)?;
    int_sym.0.add_ast(&ast)?;
    Ok(())
}

pub fn process_cmd(
    int_sym: &mut (Int, Sym),
    int_tree: &mut IntTree,
    mut cmds: impl Iterator<Item = Command> + Clone,
) -> Result {
    while let Some(cmd) = cmds.next() {
        match cmd {
            Command::Loop(n) => {
                for _ in 0..n {
                    process_cmd(int_sym, int_tree, cmds.clone())?;
                }
                break;
            }
            Command::Tags => {
                println!(
                    "{:?}\n",
                    int_tree.keys(),
                );
            }
            Command::Tag(tag) => {
                let int = std::mem::take(&mut int_sym.0);
                if !int_tree.commit(&tag, int) {
                    return Err(commands::Error::ExistedTagName(tag).into());
                }
            }
            Command::Goto(tag) => {
                if !int_tree.checkout(&tag) {
                    return Err(commands::Error::WrongTagName(tag).into());
                } else {
                    int_sym.0 = int_tree.collect_from_head().unwrap();
                }
            }
            Command::Go => {
                int_sym.1.update(&int_sym.0);
                int_sym.1.reset();
                int_sym.1.finish();
            }
            Command::Reset => {
                int_sym.0 = Int::default();
            }
            Command::Load(path) => {
                let ast = Ast::from_file(&path)?;
                int_sym.0 = Int::new(&ast)?;
            }
            Command::Class => {
                int_sym.1.update(&int_sym.0);
                println!("CReg: {}\n", int_sym.1.get_class().get())
            }
            Command::Polar => {
                int_sym.1.update(&int_sym.0);
                println!("QReg polar: {:.4?}\n", int_sym.1.get_polar_wavefunction());
            }
            Command::Probs => {
                int_sym.1.update(&int_sym.0);
                println!("QReg probabilities: {:.4?}\n", int_sym.1.get_probabilities());
            }
            Command::Ops => {
                int_sym.1.update(&int_sym.0);
                println!("Operations: {}\n", int_sym.0.get_ops_tree());
            }
            Command::Names => {
                println!("QReg: {}\nCReg: {}\n", int_sym.0.get_q_alias(), int_sym.0.get_c_alias());
            }
            Command::Help => {
                println!("{}", commands::HELP);
            }
            Command::Quit => {
                return Err(Error::Quit(0));
            }
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
