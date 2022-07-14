use std::collections::HashMap;

use crate::atom::Atom;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use gc::{Finalize, Gc, Trace};

#[derive(Clone, PartialEq, Debug, Trace, Finalize)]
pub struct Env {
    bindings: HashMap<String, Gc<Atom>>,
    parent: Option<Box<Env>>,
}

impl Default for Env {
    fn default() -> Self {
        let mut env = Self {
            bindings: Default::default(),
            parent: Default::default(),
        };

        env.set(String::from("nil"), Gc::new(Atom::nil()));
        env.set(String::from("t"), Gc::new(Atom::t()));

        env.set(String::from("define"), Gc::new(Atom::symbol("define")));
        env.set(String::from("defmacro"), Gc::new(Atom::symbol("defmacro")));
        env.set(String::from("lambda"), Gc::new(Atom::symbol("lambda")));
        env.set(String::from("if"), Gc::new(Atom::symbol("if")));
        env.set(String::from("quote"), Gc::new(Atom::symbol("quote")));
        env.set(String::from("apply"), Gc::new(Atom::symbol("apply")));

        env.add_builtin("pair?", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin pair? expected exactly one argument, got {}",
                    args
                ))
            } else if Atom::is_list(args.car()?) {
                Ok(Gc::new(Atom::t()))
            } else {
                Ok(Gc::new(Atom::nil()))
            }
        });

        env.add_builtin("symbol?", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin symbol? expected exactly one argument, got {}",
                    args
                ))
            } else if matches!(args.car()?.as_ref(), Atom::Symbol(_)) {
                Ok(Gc::new(Atom::t()))
            } else {
                Ok(Gc::new(Atom::nil()))
            }
        });

        env.add_builtin("string?", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin string? expected exactly one argument, got {}",
                    args
                ))
            } else if matches!(args.car()?.as_ref(), Atom::String(_)) {
                Ok(Gc::new(Atom::t()))
            } else {
                Ok(Gc::new(Atom::nil()))
            }
        });

        env.add_builtin("string-length", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin string-length expected exactly one argument, got {}",
                    args
                ))
            } else {
                match args.car()?.as_ref() {
                    Atom::String(s) => Ok(Gc::new(Atom::integer(s.len() as i64))),
                    a => Err(eyre!(
                        "Builtin string-length expected its argument to be a string, but got {}",
                        a
                    )),
                }
            }
        });

        env.add_builtin("car", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin car expected exactly one argument, got {}",
                    args
                ))
            } else if args.car()?.is_nil() {
                Ok(Gc::new(Atom::nil()))
            } else {
                args.car()?.car()
            }
        });

        env.add_builtin("cdr", |args| {
            if args.is_nil() || !args.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin cdr expected exactly one argument, got {}",
                    args
                ))
            } else if args.car()?.is_nil() {
                Ok(Gc::new(Atom::nil()))
            } else {
                args.car()?.cdr()
            }
        });

        env.add_builtin("cons", |args| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin cons expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let car = args.car()?;
                let cdr = args.cdr()?.car()?;
                Ok(Gc::new(Atom::Pair(car, cdr)))
            }
        });

        env.add_builtin("+", |args| {
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
                Ok(Gc::new(Atom::number(arg1 + arg2)))
            }
        });

        env.add_builtin("-", |args| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin - expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 - arg2)))
            }
        });

        env.add_builtin("*", |args| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin * expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 * arg2)))
            }
        });

        env.add_builtin("/", |args| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin / expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 / arg2)))
            }
        });

        env.add_builtin("%", |args| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin % expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car()?.get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()?
                    .car()?
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 % arg2)))
            }
        });

        env.add_builtin("=", |args| {
            if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
                Err(eyre!(
                    "Builtin = expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().context("As first argument")?;
                let arg2 = args.cdr()?.car().context("As second argument")?;
                Ok(Gc::new(Atom::bool(arg1 == arg2)))
            }
        });

        env.add_builtin("<", |args| {
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
                Ok(Gc::new(Atom::bool(arg1 < arg2)))
            }
        });

        env.add_builtin("<=", |args| {
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
                Ok(Gc::new(Atom::bool(arg1 <= arg2)))
            }
        });

        env.add_builtin(">", |args| {
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
                Ok(Gc::new(Atom::bool(arg1 > arg2)))
            }
        });

        env.add_builtin(">=", |args| {
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
                Ok(Gc::new(Atom::bool(arg1 >= arg2)))
            }
        });

        Env::new(Some(Box::new(env)))
    }
}

impl Env {
    pub fn new(parent: Option<Box<Env>>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent,
        }
    }

    pub fn get(&self, name: &str) -> Result<Gc<Atom>> {
        match self.bindings.get(name) {
            Some(atom) => Ok(atom.clone()),
            None => match &self.parent {
                Some(parent) => parent
                    .get(name)
                    .context("Trying parent environment".to_string())
                    .context(format!(
                        "Symbol {name} is not bound to any value in current environment. Bound symbols: {:?}",
                        self.bindings.keys()
                    )),
                None => Err(eyre!(format!(
                    "No parent enviroment left to try"
                )))
                .context(format!(
                    "Symbol {name} is not bound to any value in current environment. Bound symbols: {:?}",
                    self.bindings.keys()
                )),
            },
        }
    }

    pub fn set(&mut self, name: String, value: Gc<Atom>) {
        self.bindings.insert(name, value);
    }

    fn add_builtin(&mut self, name: &str, value: fn(Gc<Atom>) -> Result<Gc<Atom>>) {
        self.set(String::from(name), Gc::new(Atom::NativeFunc(value)))
    }

    pub fn add_furthest_parent(&mut self, parent: Env) {
        if self.parent.is_none() {
            self.parent = Some(Box::new(parent))
        } else {
            self.parent.as_mut().unwrap().add_furthest_parent(parent)
        }
    }
}
