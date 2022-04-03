use qvnt::qasm::Int;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub struct IntTree {
    head: RefCell<Option<Rc<String>>>,
    map: HashMap<Rc<String>, (Weak<String>, Int)>,
}

impl IntTree {
    pub fn with_root<S: ToString>(root: S) -> Self {
        Self {
            head: RefCell::new(Some(Rc::new(root.to_string()))),
            map: HashMap::new(),
        }
    }

    pub fn keys(&self) -> Vec<(Rc<String>, Rc<String>)> {
        self.map.iter().filter_map(|(a, (b, _))| {
            Some((Rc::clone(a), Weak::upgrade(b)?))
        }).collect()
    }

    pub fn commit<S: AsRef<str>>(&mut self, tag: S, change: Int) -> bool {
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

    fn find_map<F: FnMut(&String) -> Option<R>, R>(&self, start: &String, mut f: F) -> Option<R> {
        let mut start = Rc::new(start.to_string());
        loop {
            let curr = match self.map.get(&start) {
                Some(curr) => curr,
                None => break None,
            };
            if let Some(next) = Weak::upgrade(&curr.0) {
                if let Some(r) = f(&*next) {
                    break Some(r);
                }
                start = Rc::clone(&next);
            } else {
                return None;
            }
        }
    }

    fn common_commit(&self, a: &String, b: &String) -> Option<String> {
        self.find_map(a, |from_a| {
            self.find_map(b, |from_b| {
                if from_a == from_b {
                    Some(from_a.clone())
                } else {
                    None
                }
            })
        })
    }

    pub fn collect_from_head(&self) -> Option<Int> {
        let mut start = Rc::clone(self.head.borrow().as_ref()?);
        let mut int_changes = Int::default();

        loop {
            let mut curr = self.map.get(&start)?.clone();
            int_changes.add_int(&mut curr.1);
            if let Some(next) = Weak::upgrade(&curr.0) {
                start = Rc::clone(&next);
            } else {
                break Some(int_changes);
            }
        }
    }

    fn collect(&self, start: &String, end: &String) -> Option<Int> {
        let mut start = Rc::new(start.to_string());
        let mut int_changes = Int::default();
        loop {
            let mut curr = self.map.get(&start)?.clone();
            int_changes.add_int(&mut curr.1);
            if let Some(next) = Weak::upgrade(&curr.0) {
                if &*next == end {
                    break;
                }
                start = Rc::clone(&next);
            } else {
                return None;
            }
        }

        Some(int_changes)
    }

    fn route(&self, tag: &String) -> Option<(Int, Int)> {
        let head = self.head.borrow();
        let head = head.as_ref()?;
        let common_commit = self.common_commit(head.as_ref(), tag)?;
        Some((
            self.collect(head.as_ref(), &common_commit)?,
            self.collect(tag, &common_commit)?,
        ))
    }
}
