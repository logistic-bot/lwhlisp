use std::collections::HashMap;
use std::rc::Rc;

use crate::atom::Atom;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;

#[derive(Clone, PartialEq, Debug)]
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

        env.set(String::from("nil"), Rc::new(Atom::nil()));
        env.set(String::from("t"), Rc::new(Atom::t()));

        env.set(String::from("define"), Rc::new(Atom::symbol("define")));
        env.set(String::from("lambda"), Rc::new(Atom::symbol("lambda")));
        env.set(String::from("quote"), Rc::new(Atom::symbol("quote")));
        env.set(String::from("if"), Rc::new(Atom::symbol("if")));

        env.add_builtin("car", |args, _env| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin car expected exactly one argument, got {}",
                    args
                ))
            } else if args.car()?.is_nil() {
                Ok(Rc::new(Atom::nil()))
            } else {
                let car = args.car()?;
                match car.as_ref() {
                    Atom::Pair(car, _) => Ok(car.clone()),
                    a => Err(eyre!("Expected argument to car to be a list, got {}", a)),
                }
            }
        });

        env.add_builtin("cdr", |args, _env| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin cdr expected exactly one argument, got {}",
                    args
                ))
            } else if args.car()?.is_nil() {
                Ok(Rc::new(Atom::nil()))
            } else {
                let car = args.car()?;
                match car.as_ref() {
                    Atom::Pair(_, cdr) => Ok(cdr.clone()),
                    a => Err(eyre!("Expected argument to cdr to be a list, got {}", a)),
                }
            }
        });

        env.add_builtin("cons", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin cons expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let car = args.car()?;
                let cdr = args.cdr()?.car()?;
                Ok(Rc::new(Atom::Pair(car, cdr)))
            }
        });

        env.add_builtin("+", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin + expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::number(arg1 + arg2)))
            }
        });

        env.add_builtin("-", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin + expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::number(arg1 - arg2)))
            }
        });

        env.add_builtin("*", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin + expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::number(arg1 * arg2)))
            }
        });

        env.add_builtin("/", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin + expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::number(arg1 / arg2)))
            }
        });

        env.add_builtin("%", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin + expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::number(arg1 % arg2)))
            }
        });

        env.add_builtin("=", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin = expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().context("As first argument")?;
                let arg2 = args.cdr()?.car().context("As second argument")?;
                Ok(Rc::new(Atom::bool(arg1 == arg2)))
            }
        });

        env.add_builtin("<", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin < expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::bool(arg1 < arg2)))
            }
        });

        env.add_builtin("<=", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin <= expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::bool(arg1 <= arg2)))
            }
        });

        env.add_builtin(">", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin > expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::bool(arg1 > arg2)))
            }
        });

        env.add_builtin(">=", |args, _env| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin >= expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Rc::new(Atom::bool(arg1 >= arg2)))
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

    fn add_builtin(&mut self, name: &str, value: fn(Rc<Atom>, &mut Env) -> Result<Rc<Atom>>) {
        self.set(String::from(name), Rc::new(Atom::NativeFunc(value)))
    }

    pub fn add_furthest_parent(&mut self, parent: Env) {
        if self.parent.is_none() {
            self.parent = Some(Box::new(parent))
        } else {
            self.parent.as_mut().unwrap().add_furthest_parent(parent)
        }
    }
}
