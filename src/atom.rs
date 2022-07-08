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
    Macro(Env, Rc<Atom>, Rc<Atom>),
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Pair(l0, l1), Self::Pair(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::NativeFunc(_), Self::NativeFunc(_)) => false,
            (Self::Closure(l0, l1, l2), Self::Closure(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            _ => false,
        }
    }
}

impl std::fmt::Debug for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <dyn std::fmt::Display>::fmt(self, f)
    }
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
            Atom::Macro(_env, args, expr) => write!(f, "(defmacro {} {})", args, expr),
        }
    }
}

impl Atom {
    pub fn car(&self) -> Result<Rc<Atom>> {
        match self {
            Atom::Pair(car, _) => Ok(car.clone()),
            Atom::Symbol(name) if name.as_str() == "nil" => Ok(Rc::new(Atom::nil())),
            a => Ok(Rc::new(a.clone())),
        }
    }

    pub fn cdr(&self) -> Result<Rc<Atom>> {
        match self {
            Atom::Pair(_, cdr) => Ok(cdr.clone()),
            Atom::Symbol(name) if name.as_str() == "nil" => Ok(Rc::new(Atom::nil())),
            a => Ok(Rc::new(a.clone())),
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

    pub fn is_list(expr: Rc<Self>) -> bool {
        matches!(expr.as_ref(), Atom::Pair(_, _))
    }

    pub fn nil() -> Atom {
        Atom::symbol("nil")
    }

    pub fn t() -> Atom {
        Atom::symbol("t")
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

    fn validate_closure_form(
        env: Env,
        args: Rc<Atom>,
        body: Rc<Atom>,
    ) -> Result<(Env, Rc<Atom>, Rc<Atom>)> {
        if !Atom::is_proper_list(body.clone()) {
            Err(eyre!("Expected body to be a proper list, got {body}"))
        } else {
            // check argument names are all symbol
            let mut p = args.clone();
            while !p.is_nil() {
                match p.as_ref() {
                        Atom::Symbol(_) => break,
                        Atom::Pair(car, cdr) => {
                            if !matches!(car.as_ref(), Atom::Symbol(_)) {
                                return Err(eyre!("Expected all argument names to be symbols, but got {}, which is not a symbol", car))
                            }
                            p = cdr.clone();
                        },
                        a => return Err(eyre!("Expected all argument names to be symbols, but got {}, which is not a symbol", a))
                    }
            }

            Ok((env, args, body))
        }
    }

    pub fn closure(env: Env, args: Rc<Atom>, body: Rc<Atom>) -> Result<Rc<Atom>> {
        let (env, args, body) = Atom::validate_closure_form(env, args, body)?;
        Ok(Rc::new(Atom::Closure(env, args, body)))
    }

    pub fn closure_add_env_binding(
        atom: Rc<Atom>,
        name: String,
        value: Rc<Atom>,
    ) -> Result<Rc<Atom>> {
        match atom.as_ref() {
            Atom::Closure(env, a, b) => {
                let mut env = env.clone();
                env.set(name, value);
                Ok(Rc::new(Atom::Closure(env, a.clone(), b.clone())))
            }
            a => {
                Err(eyre!(format!("Tried to change the environment of a closure, but the provided atom was not a closure. Found {}", a)))
            }
        }
    }

    pub fn as_bool(&self) -> bool {
        !self.is_nil()
    }

    pub fn bool(b: bool) -> Self {
        if b {
            Atom::t()
        } else {
            Atom::nil()
        }
    }

    pub fn get_list_item_by_index(list: Rc<Self>, index: usize) -> Result<Rc<Self>> {
        let mut list = list;
        let mut index = index;
        while index > 0 {
            index -= 1;
            list = list.cdr()?;
        }
        list.car()
    }
}
