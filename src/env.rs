use std::collections::HashMap;
use std::rc::Rc;

use crate::atom::Atom;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use gc::{Finalize, Gc, Trace};
use tracing::trace;
use tracing::{info, instrument};

/// This holds bindings from symbols to atoms.
#[derive(Clone, PartialEq, Debug, Trace, Finalize)]
pub struct Env {
    bindings: HashMap<Rc<String>, Gc<Atom>>,
    parent: Option<Box<Env>>,
}

impl Default for Env {
    #[instrument]
    fn default() -> Self {
        info!("Creating new default Env");
        let mut env = Self {
            bindings: HashMap::new(),
            parent: None,
        };

        env.set(String::from("nil"), Gc::new(Atom::nil()));
        env.set(String::from("t"), Gc::new(Atom::t()));

        env.set(String::from("define"), Gc::new(Atom::symbol("define")));
        env.set(String::from("defmacro"), Gc::new(Atom::symbol("defmacro")));
        env.set(String::from("lambda"), Gc::new(Atom::symbol("lambda")));
        env.set(String::from("if"), Gc::new(Atom::symbol("if")));
        env.set(String::from("quote"), Gc::new(Atom::symbol("quote")));
        env.set(String::from("apply"), Gc::new(Atom::symbol("apply")));

        env.add_builtin("into-pretty-string", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin into-pretty-string expected exactly one argument, got {}",
                    args
                ))
            } else {
                let arg = args.car();
                let s = format!("{}", arg);
                Ok(Gc::new(Atom::String(s)))
            }
        });

        env.add_builtin("into-string", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin into-string expected exactly one argument, got {}",
                    args
                ))
            } else {
                let arg = args.car();
                let a = arg.as_ref();
                let s = format!("{:?}", a);
                Ok(Gc::new(Atom::String(s)))
            }
        });

        env.add_builtin("print", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin print expected exactly one argument, got {}",
                    args
                ))
            } else {
                let arg = args.car();
                let s = format_for_print(&arg);
                print!("{}", &s);
                Ok(Gc::new(Atom::String(s)))
            }
        });

        env.add_builtin("println", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin println expected exactly one argument, got {}",
                    args
                ))
            } else {
                let arg = args.car();
                let s = format_for_print(&arg);
                println!("{}", &s);
                Ok(Gc::new(Atom::String(s)))
            }
        });

        env.add_builtin("pair?", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin pair? expected exactly one argument, got {}",
                    args
                ))
            } else if Atom::is_list(&args.car()) {
                Ok(Gc::new(Atom::t()))
            } else {
                Ok(Gc::new(Atom::nil()))
            }
        });

        env.add_builtin("symbol?", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin symbol? expected exactly one argument, got {}",
                    args
                ))
            } else if matches!(args.car().as_ref(), Atom::Symbol(_)) {
                Ok(Gc::new(Atom::t()))
            } else {
                Ok(Gc::new(Atom::nil()))
            }
        });

        env.add_builtin("string?", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin string? expected exactly one argument, got {}",
                    args
                ))
            } else if matches!(args.car().as_ref(), Atom::String(_)) {
                Ok(Gc::new(Atom::t()))
            } else {
                Ok(Gc::new(Atom::nil()))
            }
        });

        env.add_builtin("string-length", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin string-length expected exactly one argument, got {}",
                    args
                ))
            } else {
                match args.car().as_ref() {
                    Atom::String(s) => Ok(Gc::new(Atom::integer(s.len() as i64))),
                    a => Err(eyre!(
                        "Builtin string-length expected its argument to be a string, but got {}",
                        a
                    )),
                }
            }
        });

        env.add_builtin("car", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin car expected exactly one argument, got {}",
                    args
                ))
            } else {
                Ok(args.car().car())
            }
        });

        env.add_builtin("cdr", |args| {
            if args.is_nil() || !args.cdr().is_nil() {
                Err(eyre!(
                    "Builtin cdr expected exactly one argument, got {}",
                    args
                ))
            } else {
                Ok(args.car().cdr())
            }
        });

        env.add_builtin("cons", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin cons expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let car = args.car();
                let cdr = args.cdr().car();
                Ok(Gc::new(Atom::Pair(car, cdr)))
            }
        });

        env.add_builtin("+", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin + expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 + arg2)))
            }
        });

        env.add_builtin("-", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin - expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 - arg2)))
            }
        });

        env.add_builtin("*", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin * expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 * arg2)))
            }
        });

        env.add_builtin("/", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin / expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 / arg2)))
            }
        });

        env.add_builtin("%", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin % expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::number(arg1 % arg2)))
            }
        });

        env.add_builtin("=", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin = expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car();
                let arg2 = args.cdr().car();
                Ok(Gc::new(Atom::bool(arg1 == arg2)))
            }
        });

        env.add_builtin("<", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin < expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::bool(arg1 < arg2)))
            }
        });

        env.add_builtin("<=", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin <= expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::bool(arg1 <= arg2)))
            }
        });

        env.add_builtin(">", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin > expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::bool(arg1 > arg2)))
            }
        });

        env.add_builtin(">=", |args| {
            if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
                Err(eyre!(
                    "Builtin >= expected exactly two arguments, got {}",
                    args
                ))
            } else {
                let arg1 = args.car().get_number().context("As first argument")?;
                let arg2 = args
                    .cdr()
                    .car()
                    .get_number()
                    .context("As second argument")?;
                Ok(Gc::new(Atom::bool(arg1 >= arg2)))
            }
        });

        Env::new(Some(Box::new(env)))
    }
}

fn format_for_print(arg: &Gc<Atom>) -> String {
    let s = match arg.as_ref() {
        Atom::String(string) => string.clone(),
        a => {
            format!("{}", a)
        }
    };
    s
}

impl Env {
    /// Create a new empty environemnt with the give parent environment
    #[must_use]
    pub fn new(parent: Option<Box<Env>>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent,
        }
    }

    /// Get a value from the environment, trying parent environments if the key is not found.
    pub fn get(&self, name: &str) -> Result<Gc<Atom>> {
        match self.bindings.get(&Rc::new(name.to_string())) {
            Some(atom) => Ok(atom.clone()),
            None => match &self.parent {
                Some(parent) => parent.get(name),
                None => {
                    info!("Symbol {name} is not bound to any value");
                    Err(eyre!(format!("Symbol {name} is not bound to any value.")))
                }
            },
        }
    }

    /// Set a value in the environment
    pub fn set(&mut self, name: String, value: Gc<Atom>) {
        trace!("{name} is now bound to {value:?}");
        self.bindings.insert(Rc::new(name), value);
    }

    fn add_builtin(&mut self, name: &str, value: fn(Gc<Atom>) -> Result<Gc<Atom>>) {
        info!("Adding builtin {name}");
        self.set(String::from(name), Gc::new(Atom::NativeFunc(value)));
    }

    /// Add a parent environment to the outmost parent.
    pub fn add_furthest_parent(&mut self, parent: Env) {
        trace!("Adding {parent:?} as furthest parent of {self:?}");

        match &mut self.parent {
            Some(self_parent) => self_parent.add_furthest_parent(parent),
            None => self.parent = Some(Box::new(parent)),
        }
    }
}
