use qvnt::qasm::Int;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    rc::{Rc, Weak},
};

use crate::utils;

#[derive(Debug)]
pub struct IntTree<'t> {
    head: RefCell<Option<Rc<String>>>,
    map: HashMap<Rc<String>, (Weak<String>, Int<'t>)>,
}

pub enum RemoveStatus {
    Removed,
    NotFound,
    IsParent,
    IsHead,
}

impl<'t> IntTree<'t>
where
    Self: 't,
{
    pub fn with_root<S: ToString>(root: S) -> Self {
        let root = Rc::new(root.to_string());
        let map = HashMap::from([(Rc::clone(&root), (Weak::new(), Int::default()))]);
        Self {
            head: RefCell::new(Some(root)),
            map,
        }
    }

    pub fn keys(&self) -> Vec<(Rc<String>, Rc<String>)> {
        self.map
            .iter()
            .filter_map(|(a, (b, _))| Some((Rc::clone(a), Weak::upgrade(b)?)))
            .collect()
    }

    pub fn commit<S: AsRef<str>>(&mut self, tag: S, change: Int<'t>) -> bool {
        let tag = tag.as_ref().to_string();

        if self.map.contains_key(&tag) {
            return false;
        }

        let tag = Rc::new(tag);
        let old_head = match &*self.head.borrow() {
            Some(rc) => Rc::downgrade(rc),
            None => Weak::new(),
        };
        *self.head.borrow_mut() = Some(Rc::clone(&tag));
        self.map.insert(tag, (old_head, change));

        true
    }

    pub fn checkout<S: AsRef<str>>(&self, tag: S) -> bool {
        let tag = tag.as_ref().to_string();

        match self.map.get_key_value(&tag) {
            Some(entry) => {
                *self.head.borrow_mut() = Some(Rc::clone(entry.0));
                true
            }
            None => false,
        }
    }

    pub fn collect_to_head(&self) -> Option<Int<'t>> {
        let mut start = Rc::clone(self.head.borrow().as_ref()?);
        let mut int_changes = Int::<'t>::default();

        loop {
            let curr = self.map.get(&start)?.clone();
            int_changes = unsafe { int_changes.prepend_int(curr.1.clone()) };
            if let Some(next) = Weak::upgrade(&curr.0) {
                start = Rc::clone(&next);
            } else {
                break Some(int_changes);
            }
        }
    }

    pub fn remove<S: AsRef<str>>(&mut self, tag: S) -> RemoveStatus {
        let tag = tag.as_ref().to_string();

        if self.head.borrow().as_deref() == Some(&tag) {
            return RemoveStatus::IsHead;
        }

        let mut is_presented = false;
        for tags in self.map.iter() {
            if &**tags.0 == &tag {
                is_presented = true;
            }
            if let Some(par_tag) = Weak::upgrade(&tags.1 .0) {
                if &**par_tag == &tag {
                    return RemoveStatus::IsParent;
                }
            }
        }

        if !is_presented {
            return RemoveStatus::NotFound;
        }

        let removed = self.map.remove(&tag).unwrap().1;
        <Int<'t> as utils::drop_leakage::DropExt>::drop(removed);

        RemoveStatus::Removed
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    UnknownTagCmd(String),
    UnspecifiedTag,
}

impl From<Error> for crate::lines::Error {
    fn from(e: Error) -> Self {
        Self::TagError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownTagCmd(cmd) => write!(f, "Unknown Tag subcommand {cmd:?}"),
            Error::UnspecifiedTag => write!(f, "Tag name should be specified"),
        }
    }
}

impl std::error::Error for Error {}

pub const HELP: &str = "QVNT Interpreter Tag command

USAGE:
    :tag [SUBCOMMANDS...]

SUBCOMMANDS:
    ls          Show the list of previously created tags
    mk TAG      Create TAG with current state
    ch TAG      Swap current state to TAG's state
    rm TAG      Remove TAG from tree
    help|h|?    Show this reference
";

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    List,
    Create(String),
    Remove(String),
    Checkout(String),
    Reset,
    Help,
}

impl Command {
    pub fn parse_command<'a, I: Iterator<Item = &'a str>>(
        source: &mut I,
    ) -> Result<Command, Error> {
        match source.next() {
            None | Some("ls") => Ok(Command::List),
            Some("mk") => match source.next() {
                Some(arg) => Ok(Command::Create(arg.to_string())),
                None => Err(Error::UnspecifiedTag),
            },
            Some("rm") => match source.next() {
                Some(arg) => Ok(Command::Remove(arg.to_string())),
                None => Err(Error::UnspecifiedTag),
            },
            Some("ch") => match source.next() {
                Some(arg) => Ok(Command::Checkout(arg.to_string())),
                None => Err(Error::UnspecifiedTag),
            },
            Some("reset") => Ok(Command::Reset),
            Some("help" | "h" | "?") => Ok(Command::Help),
            Some(cmd) => Err(Error::UnknownTagCmd(cmd.to_string())),
        }
    }
}
