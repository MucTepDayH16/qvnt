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
        let root = Rc::new(root.to_string());
        let map = HashMap::from([
            (Rc::clone(&root), (Weak::new(), Int::default())),
            ]);
        Self {
            head: RefCell::new(Some(root)),
            map,
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

    pub fn collect_to_head(&self) -> Option<Int> {
        let mut start = Rc::clone(self.head.borrow().as_ref()?);
        let mut int_changes = Int::default();

        loop {
            let curr = self.map.get(&start)?.clone();
            unsafe { int_changes.prepend_int(curr.1.clone()) };
            if let Some(next) = Weak::upgrade(&curr.0) {
                start = Rc::clone(&next);
            } else {
                break Some(int_changes);
            }
        }
    }
}
