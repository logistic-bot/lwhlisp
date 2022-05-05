use std::rc::Rc;

use color_eyre::eyre::eyre;
use color_eyre::Result;

#[derive(Debug, Clone)]
pub enum Atom {
    Number(f64),
    Symbol(String),
    Pair(Rc<Atom>, Rc<Atom>),
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
        }
    }
}

impl Atom {
    pub fn car(&self) -> Result<Rc<Atom>> {
        match self {
            Atom::Pair(car, _) => Ok(car.clone()),
            _ => Err(eyre!("Cannot get car of {:?}", self)),
        }
    }

    pub fn cdr(&self) -> Result<Rc<Atom>> {
        match self {
            Atom::Pair(_, cdr) => Ok(cdr.clone()),
            _ => Err(eyre!("Cannot get cdr of {:?}", self)),
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Atom::Symbol(sym) => sym.as_str() == "nil",
            _ => false,
        }
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
}
