use std::rc::Rc;

use color_eyre::eyre::eyre;
use color_eyre::Result;

use crate::env::Env;

/// Evalutation happens here.
pub mod eval;

/// A single value in lwhlisp.
#[derive(Clone)]
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
    Pair(Rc<Atom>, Rc<Atom>),
    /// Native Rust function.
    ///
    /// This is used to implement some base function that require direct access to the underlying data.
    NativeFunc(fn(Rc<Atom>) -> Result<Rc<Atom>>),
    /// Closure
    Closure(Env, Rc<Atom>, Rc<Atom>),
    /// Macro
    Macro(Env, Rc<Atom>, Rc<Atom>),
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Pair(l0, l1), Self::Pair(r0, r1)) => l0 == r0 && l1 == r1,
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
                                for _ in 0..=indent_level {
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
                    Rc::new(Atom::symbol("defmacro")),
                    Rc::new(Atom::Pair(args.clone(), expr.clone())),
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
    /// Get the car of the atom if it is a pair, else return the atom itself.
    pub fn car(&self) -> Rc<Atom> {
        match self {
            Atom::Pair(car, _) => car.clone(),
            a => Rc::new(a.clone()),
        }
    }

    /// Get the cdr of the atom if it is a pair, else return the atom itself.
    pub fn cdr(&self) -> Rc<Atom> {
        match self {
            Atom::Pair(_, cdr) => cdr.clone(),
            a => Rc::new(a.clone()),
        }
    }

    /// Get the cdr of the atom if it is a pair.
    ///
    /// The cdr of nil is nil.
    ///
    /// # Errors
    /// If the atom is not a pair or nil, return an error.
    pub fn strict_cdr(&self) -> Result<Rc<Atom>> {
        if self.is_nil() {
            Ok(Rc::new(self.clone()))
        } else {
            match self {
                Atom::Pair(_, cdr) => Ok(cdr.clone()),
                _ => Err(eyre!("Tried to get cdr of {:?}, which is invalid", self)),
            }
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

    /// Return true if the atom is a pair.
    pub fn is_list(expr: &Rc<Self>) -> bool {
        matches!(expr.as_ref(), Atom::Pair(_, _))
    }

    /// Creates a nil atom
    #[must_use]
    pub fn nil() -> Atom {
        Atom::symbol("nil")
    }

    /// Creates a t atom
    #[must_use]
    pub fn t() -> Atom {
        Atom::symbol("t")
    }

    /// Constructs a pair from two atoms
    #[must_use]
    pub fn cons(car: Atom, cdr: Atom) -> Atom {
        Atom::Pair(Rc::new(car), Rc::new(cdr))
    }

    /// Constructs a symbol from a string
    #[must_use]
    pub fn symbol(sym: &str) -> Atom {
        Atom::Symbol(String::from(sym))
    }

    /// Constructs a number from a number
    #[must_use]
    pub const fn number(num: f64) -> Atom {
        Atom::Number(num)
    }

    /// Constructs a number from an integer
    ///
    /// Warning: may cause precision loss if more than 52 bits are needed to represent the given integer
    #[must_use]
    pub const fn integer(num: i64) -> Atom {
        #[allow(clippy::cast_precision_loss)]
        Atom::Number(num as f64)
    }

    /// Get the value if the atom is a number.
    ///
    /// # Errors
    /// If the given atom is not a number, return an error.
    pub fn get_number(&self) -> Result<f64> {
        match self {
            Atom::Number(x) => Ok(*x),
            a => Err(eyre!("Expected a number, got {}", a)),
        }
    }

    /// The the symbol name if the atom is a symbol, else return an error.
    ///
    /// # Errors
    /// If the given atom is not a symbol, return an error.
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
        if Atom::is_proper_list(body.clone()) {
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
        } else {
            Err(eyre!("Expected body to be a proper list, got {body}"))
        }
    }

    /// Create a closure from the given parameters
    ///
    /// # Errors
    /// Return an error if an invalid closure form is given
    pub fn closure(env: Env, args: Rc<Atom>, body: Rc<Atom>) -> Result<Rc<Atom>> {
        let (env, args, body) = Atom::validate_closure_form(env, args, body)?;
        Ok(Rc::new(Atom::Closure(env, args, body)))
    }

    /// Set a binding in a closure's environment if the atom is a closure.
    ///
    /// # Errors
    /// Returns an error if the given atom is not a closure.
    pub fn closure_add_env_binding(
        atom: &Rc<Atom>,
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

    /// Return false if the atom is nil
    pub fn as_bool(&self) -> bool {
        !self.is_nil()
    }

    /// Create nil or t from a bool
    #[must_use]
    pub fn bool(b: bool) -> Self {
        if b {
            Atom::t()
        } else {
            Atom::nil()
        }
    }

    /// Get the item of a list by index
    ///
    /// # Errors
    /// Returns an error if the given atom is not a list, or if the list is not long enough
    pub fn get_list_item_by_index(list: Rc<Self>, index: usize) -> Result<Rc<Self>> {
        let mut list = list;
        let mut index = index;
        while index > 0 {
            index -= 1;
            list = list.strict_cdr()?;
        }
        Ok(list.car())
    }

    /// WARNING: This is probably broken, and should only be used when it doesn't matter much.
    /// Currently it is used in the pretty printer, where it is used to count the lenght of a list.
    pub fn into_vec(atom: Rc<Self>) -> Vec<Rc<Self>> {
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
