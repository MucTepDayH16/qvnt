use std::{collections::HashMap, path::PathBuf};

use crate::{
    int_tree::IntTree,
    lines::{self, Command, Line},
    utils::owned_errors::ToOwnedError,
};
use qvnt::qasm::{Ast, Int, Sym};

#[derive(Debug)]
pub enum Error {
    Inner,
    #[allow(dead_code)]
    Unimplemented,
    Dyn(Box<dyn std::error::Error>),
    Quit(i32),
}

impl<E: std::error::Error + 'static> From<E> for Error {
    fn from(e: E) -> Self {
        Self::Dyn(e.into())
    }
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub struct Process<'t> {
    head: Int<'t>,
    int: Int<'t>,
    sym: Sym,
    storage: HashMap<PathBuf, Ast<'t>>,
}

impl<'t> Process<'t> {
    pub fn new(int: Int<'t>) -> Self {
        Self {
            head: Int::default(),
            int: int.clone(),
            sym: Sym::new(int),
            storage: HashMap::new(),
        }
    }

    pub fn int(&self) -> Int<'t> {
        let int = self.int.clone();
        unsafe { int.append_int(self.head.clone()) }
    }

    fn reset(&mut self, int: Int<'t>) {
        self.head = Int::default();
        self.int = int;
    }

    fn sym_update(&mut self) {
        let int = self.int();
        self.sym.init(int);
    }

    fn sym_go(&mut self) {
        self.sym_update();
        self.sym.reset();
        self.sym.finish();
    }

    pub fn process(&mut self, int_set: &mut IntTree<'t>, line: String) -> Result {
        match line.parse::<Line>() {
            Ok(Line::Qasm) => self.process_qasm(crate::program::leak_string(line, false)),
            Ok(Line::Commands(cmds)) => self.process_cmd(int_set, cmds.into_iter()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn process_qasm(&mut self, line: &'t str) -> Result {
        let ast: Ast<'t> = Ast::from_source(line).map_err(ToOwnedError::own)?;
        self.int = self
            .int
            .clone()
            .ast_changes(&mut self.head, ast)
            .map_err(ToOwnedError::own)?;
        Ok(())
    }

    pub fn process_cmd(
        &mut self,
        int_tree: &mut IntTree<'t>,
        mut cmds: impl Iterator<Item = Command> + Clone,
    ) -> Result {
        while let Some(cmd) = cmds.next() {
            match cmd {
                Command::Loop(n) => {
                    for _ in 0..n {
                        self.process_cmd(int_tree, cmds.clone())?;
                    }
                    break;
                }
                Command::Tags(tag_cmd) => self.process_tag_cmd(int_tree, tag_cmd)?,
                Command::Go => self.sym_go(),
                Command::Reset => {
                    if !int_tree.checkout("") {
                        return Err(Error::Inner);
                    }
                    self.reset(Int::default());
                }
                Command::Load(path) => self.load_qasm(int_tree, path)?,
                Command::Class => {
                    self.sym_update();
                    println!("CReg: {}\n", self.sym.get_class().get())
                }
                Command::Polar => {
                    self.sym_update();
                    println!("QReg polar: {:.4?}\n", self.sym.get_polar_wavefunction());
                }
                Command::Probs => {
                    self.sym_update();
                    println!("QReg probabilities: {:.4?}\n", self.sym.get_probabilities());
                }
                Command::Ops => println!("Operations: {}\n", self.int().get_ops_tree()),
                Command::Names => {
                    println!(
                        "QReg: {}\nCReg: {}\n",
                        self.int().get_q_alias(),
                        self.int().get_c_alias()
                    );
                }
                Command::Help => println!("{}", lines::HELP),
                Command::Quit => return Err(Error::Quit(0)),
            }
        }

        Ok(())
    }

    pub fn process_tag_cmd(
        &mut self,
        int_tree: &mut IntTree<'t>,
        tag_cmd: crate::int_tree::Command,
    ) -> Result {
        use crate::int_tree::Command;
        match tag_cmd {
            Command::List => println!("{:?}\n", int_tree.keys()),
            Command::Create(tag) => {
                if !int_tree.commit(&tag, self.head.clone()) {
                    return Err(lines::Error::ExistedTagName(tag).into());
                } else {
                    unsafe {
                        self.int = self.int.clone().append_int(std::mem::take(&mut self.head))
                    };
                }
            }
            Command::Remove(tag) => {
                if !int_tree.remove(&tag) {
                    return Err(lines::Error::TagIsParent(tag).into());
                }
            }
            Command::Checkout(tag) => {
                if int_tree.checkout(&tag) {
                    return Err(lines::Error::WrongTagName(tag).into());
                } else {
                    let new_int = int_tree.collect_to_head().ok_or(Error::Inner)?;
                    self.reset(new_int);
                }
            }
            Command::Help => println!("{}", crate::int_tree::HELP),
        }
        Ok(())
    }

    pub fn load_qasm(&mut self, int_tree: &mut IntTree<'t>, path: PathBuf) -> Result {
        let path_tag = format!("file://{}", path.display());
        if int_tree.checkout(&path_tag) {
            self.reset(int_tree.collect_to_head().ok_or(Error::Inner)?);
        } else {
            let default_ast = {
                let source = std::fs::read_to_string(path.clone())?;
                let source = crate::program::leak_string(source, false);
                eprintln!(
                    "Leakage {{ ptr: {:?}, len: {} }}",
                    source as *const _,
                    source.len()
                );
                Ast::from_source(source).map_err(ToOwnedError::own)?
            };
            let ast = self.storage.entry(path).or_insert(default_ast).clone();
            int_tree.checkout("");
            let int = Int::new(ast).map_err(ToOwnedError::own)?;
            if !int_tree.commit(&path_tag, int.clone()) {
                return Err(Error::Inner);
            }
            self.reset(int);
        }
        Ok(())
    }
}
