use std::rc::Rc;

use color_eyre::eyre::eyre;
use color_eyre::Result;

use crate::env::Env;

pub mod eval;

#[derive(Clone)]
pub enum Atom {
    Number(f64),
    Symbol(String),
    Pair(Rc<Atom>, Rc<Atom>),
    NativeFunc(fn(Rc<Atom>, &mut Env) -> Result<Rc<Atom>>),
    Closure(Env, Rc<Atom>, Rc<Atom>),
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Number(i) => write!(f, "{}", i),
            Atom::Symbol(s) => write!(f, "{}", s),
            Atom::Pair(car, cdr) => {
                write!(f, "(")?;
                write!(f, "{}", car)?;
                let mut atom = cdr;
                while !atom.is_nil() {
                    match atom.as_ref() {
                        Atom::Pair(car, cdr) => {
                            write!(f, " {}", car)?;
                            atom = cdr;
                        }
                        a => {
                            write!(f, " . {}", a)?;
                            break;
                        }
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
            Atom::NativeFunc(_) => write!(f, "#<BUILTIN>"),
            Atom::Closure(_env, args, expr) => write!(f, "(lambda {} {})", args, expr),
        }
    }
}

impl Atom {
    pub fn car(&self) -> Result<Rc<Atom>> {
        match self {
            Atom::Pair(car, _) => Ok(car.clone()),
            Atom::Symbol(name) => {
                if name.as_str() == "nil" {
                    Ok(Rc::new(Atom::nil()))
                } else {
                    Err(eyre!("Cannot get car of {}", self))
                }
            }
            _ => Err(eyre!("Cannot get car of {}", self)),
        }
    }

    pub fn cdr(&self) -> Result<Rc<Atom>> {
        match self {
            Atom::Pair(_, cdr) => Ok(cdr.clone()),
            Atom::Symbol(name) => {
                if name.as_str() == "nil" {
                    Ok(Rc::new(Atom::nil()))
                } else {
                    Err(eyre!("Cannot get car of {}", self))
                }
            }
            _ => Err(eyre!("Cannot get cdr of {}", self)),
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Atom::Symbol(sym) => sym.as_str() == "nil",
            _ => false,
        }
    }

    pub fn is_proper_list(expr: Rc<Self>) -> bool {
        let mut expr = expr;
        while !expr.is_nil() {
            match expr.as_ref() {
                Atom::Pair(_car, cdr) => expr = cdr.clone(),
                _ => return false,
            }
        }

        true
    }

    pub fn nil() -> Atom {
        Atom::symbol("nil")
    }

    pub fn cons(car: Atom, cdr: Atom) -> Atom {
        Atom::Pair(Rc::new(car), Rc::new(cdr))
    }

    pub fn symbol(sym: &str) -> Atom {
        Atom::Symbol(String::from(sym))
    }

    pub fn number(num: f64) -> Atom {
        Atom::Number(num)
    }

    pub fn integer(num: i64) -> Atom {
        Atom::Number(num as f64)
    }

    pub fn get_number(&self) -> Result<f64> {
        match self {
            Atom::Number(x) => Ok(*x),
            a => Err(eyre!("Expected a number, got {}", a)),
        }
    }

    pub fn get_symbol_name(&self) -> Result<String> {
        match self {
            Atom::Symbol(name) => Ok(name.clone()),
            a => Err(eyre!("Expected a symbol, got {}", a)),
        }
    }

    pub fn closure(env: Env, args: Rc<Atom>, body: Rc<Atom>) -> Result<Rc<Atom>> {
        if !Atom::is_proper_list(args.clone()) {
            Err(eyre!("Expected arguments to be a proper list, got {args}"))
        } else if !Atom::is_proper_list(body.clone()) {
            Err(eyre!("Expected body to be a proper list, got {body}"))
        } else {
            // check argument names are all symbol
            let mut p = args.clone();
            while !p.is_nil() {
                match p.car()?.as_ref() {
                        Atom::Symbol(_) => (),
                        a => return Err(eyre!("Expected all argument names to be symbols, but got {a}, which is not a symbol"))
                    }
                p = p.cdr()?;
            }

            Ok(Rc::new(Atom::Closure(env, args, body)))
        }
    }
}
