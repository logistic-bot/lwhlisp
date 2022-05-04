use std::rc::Rc;

use color_eyre::eyre::eyre;
use color_eyre::Result;

#[derive(Debug, Clone)]
pub enum Atom {
    Integer(i64),
    Symbol(String),
    Pair(Rc<Atom>, Rc<Atom>),
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

    pub fn integer(int: i64) -> Atom {
        Atom::Integer(int)
    }
}
