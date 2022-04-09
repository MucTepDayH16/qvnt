use std::path::PathBuf;

use crate::{
    int_tree::IntTree,
    lines::{self, Command, Line},
};
use qvnt::qasm::{Ast, Int, Sym};

#[derive(Debug)]
pub enum Error {
    Inner,
    Dyn(Box<dyn std::error::Error>),
    Quit(i32),
}

impl<E: std::error::Error + 'static> From<E> for Error {
    fn from(e: E) -> Self {
        Self::Dyn(e.into())
    }
}

type Result = std::result::Result<(), Error>;

pub struct Process {
    head: Int,
    int: Int,
    sym: Sym,
}

impl Process {
    pub fn new(int: Int) -> Self {
        Self {
            head: Int::default(),
            int: int.clone(),
            sym: Sym::new(int),
        }
    }

    pub fn int(&self) -> Int {
        let mut int = self.int.clone();
        unsafe { int.append_int(self.head.clone()) };
        int
    }

    fn reset(&mut self, int: Int) {
        self.head = Int::default();
        self.int = int;
    }

    fn add_ast(&mut self, ast: &Ast) -> Result {
        self.int.ast_changes(&mut self.head, ast)?;
        Ok(())
    }

    fn sym_update(&mut self) {
        self.sym.init(self.int());
    }

    fn sym_go(&mut self) {
        self.sym_update();
        self.sym.reset();
        self.sym.finish();
    }
}

pub fn file_tag(path: &PathBuf) -> String {
    format!("file://{}", path.display())
}

pub fn handle_error(result: Result, dbg: bool) -> Option<i32> {
    match result {
        Ok(()) => None,
        Err(Error::Inner) => {
            eprintln!("Internal Error: Please report this to the developer.");
            Some(0xDE)
        }
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

pub fn process<'a>(curr_process: &mut Process, int_set: &mut IntTree, line: String) -> Result {
    match line.parse::<Line>() {
        Ok(Line::Qasm) => process_qasm(curr_process, line),
        Ok(Line::Commands(cmds)) => process_cmd(curr_process, int_set, cmds.into_iter()),
        Err(err) => Err(err.into()),
    }
}

pub fn process_qasm(curr_process: &mut Process, line: String) -> Result {
    let ast = Ast::from_source(line)?;
    curr_process.add_ast(&ast)
}

pub fn process_cmd(
    curr_process: &mut Process,
    int_tree: &mut IntTree,
    mut cmds: impl Iterator<Item = Command> + Clone,
) -> Result {
    while let Some(cmd) = cmds.next() {
        match cmd {
            Command::Loop(n) => {
                for _ in 0..n {
                    process_cmd(curr_process, int_tree, cmds.clone())?;
                }
                break;
            }
            Command::Tags => {
                println!("{:?}\n", int_tree.keys(),);
            }
            Command::Tag(tag) => {
                if !int_tree.commit(&tag, curr_process.head.clone()) {
                    return Err(lines::Error::ExistedTagName(tag).into());
                } else {
                    unsafe {
                        curr_process
                            .int
                            .append_int(std::mem::take(&mut curr_process.head))
                    };
                }
            }
            Command::Goto(tag) => {
                if !int_tree.checkout(&tag) {
                    return Err(lines::Error::WrongTagName(tag).into());
                } else {
                    let new_int = int_tree.collect_to_head().ok_or(Error::Inner)?;
                    curr_process.reset(new_int);
                }
            }
            Command::Go => {
                curr_process.sym_go();
            }
            Command::Reset => {
                if !int_tree.checkout("") {
                    return Err(Error::Inner);
                }
                curr_process.reset(Int::default());
            }
            Command::Load(path) => {
                let path_tag = file_tag(&path);
                if int_tree.checkout(&path_tag) {
                    curr_process.reset(int_tree.collect_to_head().ok_or(Error::Inner)?);
                } else {
                    let ast = Ast::from_file(&path, None)?;
                    int_tree.checkout("");
                    let int = Int::new(&ast)?;
                    if !int_tree.commit(&path_tag, int.clone()) {
                        return Err(Error::Inner);
                    }
                    curr_process.reset(int);
                }
            }
            Command::Class => {
                curr_process.sym_update();
                println!("CReg: {}\n", curr_process.sym.get_class().get())
            }
            Command::Polar => {
                curr_process.sym_update();
                println!(
                    "QReg polar: {:.4?}\n",
                    curr_process.sym.get_polar_wavefunction()
                );
            }
            Command::Probs => {
                curr_process.sym_update();
                println!(
                    "QReg probabilities: {:.4?}\n",
                    curr_process.sym.get_probabilities()
                );
            }
            Command::Ops => {
                println!("Operations: {}\n", curr_process.int().get_ops_tree());
            }
            Command::Names => {
                println!(
                    "QReg: {}\nCReg: {}\n",
                    curr_process.int().get_q_alias(),
                    curr_process.int().get_c_alias()
                );
            }
            Command::Help => {
                println!("{}", lines::HELP);
            }
            Command::Quit => {
                return Err(Error::Quit(0));
            }
        }
    }

    Ok(())
}
