use std::collections::HashMap;
use std::rc::Rc;

use crate::atom::Atom;
use color_eyre::eyre::eyre;
use color_eyre::Result;

pub struct Env {
    bindings: HashMap<String, Rc<Atom>>,
    parent: Option<Box<Env>>,
}

impl Default for Env {
    fn default() -> Self {
        let mut env = Self {
            bindings: Default::default(),
            parent: Default::default(),
        };

        env.add_builtin("car", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin car expected exactly one argument, got {}",
                    args
                ))
            } else if args.car()?.is_nil() {
                Ok(Rc::new(Atom::nil()))
            } else {
                match args.car()?.as_ref() {
                    Atom::Pair(car, _) => Ok(car.clone()),
                    a => Err(eyre!("Expected argument to car to be a list, got {}", a)),
                }
            }
        });

        env.add_builtin("cdr", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin cdr expected exactly one argument, got {}",
                    args
                ))
            } else if args.car()?.is_nil() {
                Ok(Rc::new(Atom::nil()))
            } else {
                match args.car()?.as_ref() {
                    Atom::Pair(_, cdr) => Ok(cdr.clone()),
                    a => Err(eyre!("Expected argument to cdr to be a list, got {}", a)),
                }
            }
        });

        env
    }
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

    fn add_builtin(&mut self, name: &str, value: fn(Rc<Atom>) -> Result<Rc<Atom>>) {
        self.set(String::from(name), Rc::new(Atom::NativeFunc(value)))
    }
}
