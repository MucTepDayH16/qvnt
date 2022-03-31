use qvnt::qasm::Int;
use std::collections::HashMap;

#[derive(Debug)]
pub struct IntSet {
    pub head: String,
    pub root: String,
    pub map: HashMap<(String, String), Int>,
}

impl IntSet {
    pub fn with_root<S: ToString>(root: S) -> Self {
        let root = root.to_string();
        Self {
            head: root.clone(),
            root,
            map: HashMap::new(),
        }
    }

    pub fn tag<S: ToString>(&mut self, name: S, int: &Int) -> Option<Int> {
        let name = name.to_string();
        let head = std::mem::replace(&mut self.head, name.clone());
        self.map.insert((name, head), int.clone())
    }

    pub fn goto<S: ToString>(&mut self, name: S) -> Option<&Int> {
        let name = name.to_string();
        let ((this, _), int) = self.map.iter().find(|((this, _), _)| this == &name)?;
        self.head = this.clone();
        Some(int)
    }
}
