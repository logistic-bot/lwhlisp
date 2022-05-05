use std::collections::HashMap;
use std::rc::Rc;

use crate::atom::Atom;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;

struct Env {
    bindings: HashMap<String, Rc<Atom>>,
    parent: Option<Box<Env>>,
}

impl Env {
    pub fn new(parent: Option<Box<Env>>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent,
        }
    }

    pub fn get(&self, name: &str) -> Option<Rc<Atom>> {
        match self.bindings.get(name) {
            Some(atom) => Some(atom.clone()),
            None => match &self.parent {
                Some(parent) => parent.get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, value: Rc<Atom>) {
        self.bindings.insert(name, value);
    }
}
