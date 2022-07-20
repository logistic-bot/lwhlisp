use color_eyre::eyre::eyre;
use color_eyre::Result;

use gc::{Finalize, Gc, Trace};

use crate::env::Env;

/// Evalutation happens here.
pub mod eval;

/// A single value in lwhlisp.
#[derive(Clone, Trace, Finalize)]
pub enum Atom {
    /// Number
    Number(f64),
    /// String
    String(String),
    /// Symbol
    Symbol(String),
    /// Pair.
    ///
    /// This is also used to construct lists, using nested pairs.
    /// For example (pseudocode), `Pair(1, Pair(2, Pair(3, nil)))` would be interpreted as `(1 2 3)`.
    Pair(Gc<Atom>, Gc<Atom>),
    /// Native Rust function.
    ///
    /// This is used to implement some base function that require direct access to the underlying data.
    NativeFunc(fn(Gc<Atom>) -> Result<Gc<Atom>>),
    /// Closure
    Closure(Env, Gc<Atom>, Gc<Atom>),
    /// Macro
    Macro(Env, Gc<Atom>, Gc<Atom>),
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

impl Atom {
    fn fmt_pair_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Pair(car, cdr) => {
                write!(f, "{:?}", car)?;
                let mut atom = cdr;
                while !atom.is_nil() {
                    match atom.as_ref() {
                        Atom::Pair(car, cdr) => {
                            write!(f, " {:?}", car)?;
                            atom = cdr;
                        }
                        a => {
                            write!(f, " . {:?}", a)?;
                            break;
                        }
                    }
                }
                Ok(())
            }
            _ => Err(std::fmt::Error {}),
        }
    }
}

impl std::fmt::Debug for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Number(i) => write!(f, "{}", i),
            Atom::Symbol(s) => write!(f, "{}", s),
            Atom::Pair(_, _) => {
                write!(f, "(")?;
                self.fmt_pair_debug(f)?;
                write!(f, ")")?;
                Ok(())
            }
            Atom::NativeFunc(_) => write!(f, "#<BUILTIN>"),
            Atom::Closure(_env, args, expr) => {
                write!(f, "(lambda {:?} ", args)?;
                expr.fmt_pair_debug(f)?;
                write!(f, ")")
            }
            Atom::Macro(_env, args, expr) => {
                write!(f, "(defmacro {:?} ", args)?;
                expr.fmt_pair_debug(f)?;
                write!(f, ")")
            }
            Atom::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_print(0))
    }
}

impl Atom {
    fn pretty_print(&self, indent_level: usize) -> String {
        use std::fmt::Write as _;

        match self {
            Atom::Pair(car, cdr) if self.get_list_lenght_including_inner() <= 12 => {
                let mut s = String::new();
                s.push('(');

                write!(s, "{}", car).unwrap();
                let mut atom = cdr;
                while !atom.is_nil() {
                    match atom.as_ref() {
                        Atom::Pair(car, cdr) => {
                            write!(s, " {}", car).unwrap();
                            atom = cdr;
                        }
                        a => {
                            write!(s, " . {}", a).unwrap();
                            break;
                        }
                    }
                }

                s.push(')');
                s
            }
            Atom::Pair(car, cdr) => {
                let mut s = String::new();
                s.push('(');

                write!(s, "{}", car.pretty_print(indent_level + 1)).unwrap();
                let mut atom = cdr;
                let mut print_on_first_line = false;
                let mut first_arg = true;
                if let Atom::Symbol(sym) = car.as_ref() {
                    if matches!(sym.as_str(), "if" | "define" | "defmacro" | "lambda") {
                        print_on_first_line = true;
                    }
                }
                while !atom.is_nil() {
                    match atom.as_ref() {
                        Atom::Pair(car, cdr) => {
                            if print_on_first_line && first_arg {
                                write!(s, " {}", car.pretty_print(indent_level + 1)).unwrap();
                            } else {
                                writeln!(s).unwrap();
                                for _ in 0..indent_level + 1 {
                                    write!(s, "   ").unwrap();
                                }
                                write!(s, "{}", car.pretty_print(indent_level + 1)).unwrap();
                            }
                            atom = cdr;
                        }
                        a => {
                            write!(s, " . {}", a).unwrap();
                            break;
                        }
                    }
                    first_arg = false;
                }

                s.push(')');
                s
            }
            Atom::Macro(_env, args, expr) => {
                let mut s = String::new();
                let atom = Atom::Pair(
                    Gc::new(Atom::symbol("defmacro")),
                    Gc::new(Atom::Pair(args.clone(), expr.clone())),
                );
                write!(s, "{}", atom.pretty_print(indent_level)).unwrap();
                s
            }
            a => {
                format!("{:?}", a)
            }
        }
    }
}

impl Atom {
    /// Get the car of the atom if it is a pair, else return an error.
    ///
    /// The car of nil is nil.
    pub fn car(&self) -> Result<Gc<Atom>> {
        match self {
            Atom::Pair(car, _) => Ok(car.clone()),
            Atom::Symbol(name) if name.as_str() == "nil" => Ok(Gc::new(Atom::nil())),
            a => Ok(Gc::new(a.clone())),
        }
    }

    /// Get the cdr of the atom if it is a pair, else return an error.
    ///
    /// The cdr of nil is nil.
    pub fn cdr(&self) -> Result<Gc<Atom>> {
        match self {
            Atom::Pair(_, cdr) => Ok(cdr.clone()),
            Atom::Symbol(name) if name.as_str() == "nil" => Ok(Gc::new(Atom::nil())),
            a => Ok(Gc::new(a.clone())),
        }
    }

    /// Returns true if the atom is nil. False otherwise
    pub fn is_nil(&self) -> bool {
        match self {
            Atom::Symbol(sym) => sym.as_str() == "nil",
            _ => false,
        }
    }

    /// Return true if the atom is a proper list.
    ///
    /// A proper list is a cons list where the last element is nil.
    pub fn is_proper_list(expr: Gc<Self>) -> bool {
        let mut expr = expr;
        while !expr.is_nil() {
            match expr.as_ref() {
                Atom::Pair(_car, cdr) => expr = cdr.clone(),
                _ => return false,
            }
        }

        true
    }

    /// Return true if the atom is a pair.
    pub fn is_list(expr: Gc<Self>) -> bool {
        matches!(expr.as_ref(), Atom::Pair(_, _))
    }

    /// Creates a nil atom
    pub fn nil() -> Atom {
        Atom::symbol("nil")
    }

    /// Creates a t atom
    pub fn t() -> Atom {
        Atom::symbol("t")
    }

    /// Constructs a pair from two atoms
    pub fn cons(car: Atom, cdr: Atom) -> Atom {
        Atom::Pair(Gc::new(car), Gc::new(cdr))
    }

    /// Constructs a symbol from a string
    pub fn symbol(sym: &str) -> Atom {
        Atom::Symbol(String::from(sym))
    }

    /// Constructs a number from a number
    pub fn number(num: f64) -> Atom {
        Atom::Number(num)
    }

    /// Constructs a number from an integer
    pub fn integer(num: i64) -> Atom {
        Atom::Number(num as f64)
    }

    /// Get the value if the atom is a number, else return an error.
    pub fn get_number(&self) -> Result<f64> {
        match self {
            Atom::Number(x) => Ok(*x),
            a => Err(eyre!("Expected a number, got {}", a)),
        }
    }

    /// The the symbol name if the atom is a symbol, else return an error.
    pub fn get_symbol_name(&self) -> Result<String> {
        match self {
            Atom::Symbol(name) => Ok(name.clone()),
            a => Err(eyre!("Expected a symbol, got {}", a)),
        }
    }

    fn validate_closure_form(
        env: Env,
        args: Gc<Atom>,
        body: Gc<Atom>,
    ) -> Result<(Env, Gc<Atom>, Gc<Atom>)> {
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

    /// Create a closure from the given parameters
    pub fn closure(env: Env, args: Gc<Atom>, body: Gc<Atom>) -> Result<Gc<Atom>> {
        let (env, args, body) = Atom::validate_closure_form(env, args, body)?;
        Ok(Gc::new(Atom::Closure(env, args, body)))
    }

    /// Set a binding in a closure's environment if the atom is a closure, return an error otherwise.
    pub fn closure_add_env_binding(
        atom: Gc<Atom>,
        name: String,
        value: Gc<Atom>,
    ) -> Result<Gc<Atom>> {
        match atom.as_ref() {
            Atom::Closure(env, a, b) => {
                let mut env = env.clone();
                env.set(name, value);
                Ok(Gc::new(Atom::Closure(env, a.clone(), b.clone())))
            }
            a => {
                Err(eyre!(format!("Tried to change the environment of a closure, but the provided atom was not a closure. Found {}", a)))
            }
        }
    }

    /// Return false if the atom is nil
    pub fn as_bool(&self) -> bool {
        !self.is_nil()
    }

    /// Create nil or t from a bool
    pub fn bool(b: bool) -> Self {
        if b {
            Atom::t()
        } else {
            Atom::nil()
        }
    }

    /// Get the item of a list by index
    pub fn get_list_item_by_index(list: Gc<Self>, index: usize) -> Result<Gc<Self>> {
        let mut list = list;
        let mut index = index;
        while index > 0 {
            index -= 1;
            list = list.cdr()?;
        }
        list.car()
    }

    /// WARNING: This is probably broken, and should only be used when it doesn't matter much.
    /// Currently it is used in the pretty printer, where it is used to count the lenght of a list.
    pub fn into_vec(atom: Gc<Self>) -> Vec<Gc<Self>> {
        match atom.as_ref() {
            Atom::Pair(car, cdr) => {
                let mut v = vec![car.clone()];
                v.append(&mut Self::into_vec(cdr.clone()));
                v
            }
            _ => {
                vec![atom]
            }
        }
    }

    /// Get length of list including sublists, or length of string if atom is a string.
    pub fn get_list_lenght_including_inner(&self) -> usize {
        match self {
            Atom::Pair(car, cdr) => {
                car.get_list_lenght_including_inner_without_symbol()
                    + cdr.get_list_lenght_including_inner_without_symbol()
            }
            Atom::Symbol(s) => s.len(),
            _ => 1,
        }
    }

    /// Get length of list including sublists.
    pub fn get_list_lenght_including_inner_without_symbol(&self) -> usize {
        match self {
            Atom::Pair(car, cdr) => {
                car.get_list_lenght_including_inner_without_symbol()
                    + cdr.get_list_lenght_including_inner_without_symbol()
            }
            _ => 1,
        }
    }
}
