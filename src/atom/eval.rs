use std::rc::Rc;

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};

use super::Atom;
use crate::env::Env;

impl Atom {
    pub fn eval(expr: Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>> {
        match expr.as_ref() {
            Atom::Number(_) => Ok(expr),
            Atom::Symbol(symbol) => Ok(env
                .get(symbol)
                .ok_or_else(|| eyre!("Symbol {} is not bound to any value", symbol))?),
            Atom::Pair(car, cdr) => {
                if Atom::is_proper_list(expr.clone()) {
                    let op = car;
                    let args = cdr;

                    match op.as_ref() {
                        Atom::Symbol(symbol) => {
                            match symbol.as_str() {
                                "quote" => {
                                    // exactly one argument
                                    if args.is_nil()
                                        || !args
                                            .cdr()
                                            .context("expected args to be a list")?
                                            .is_nil()
                                    {
                                        Err(eyre!(
                                            "QUOTE takes exactly one argument, got {}",
                                            &args
                                        ))
                                    } else {
                                        args.car()
                                    }
                                }
                                "define" => {
                                    // exactly two arguments
                                    if args.is_nil()
                                        || args.cdr()?.is_nil()
                                        || !args.cdr()?.cdr()?.is_nil()
                                    {
                                        Err(eyre!(
                                            "DEFINE takes exactly two arguments, got {}",
                                            &args
                                        ))
                                    } else {
                                        let sym = args.car()?;
                                        match sym.as_ref() {
                                            Atom::Symbol(symbol) => {
                                                let value = Atom::eval(args.cdr()?.car()?, env).context("While evaluating VALUE argument for DEFINE")?;
                                                env.set(symbol.to_string(), value);
                                                Ok(sym)
                                            },
                                            _ => Err(eyre!("Expected a symbol as first argument to define, got {}", sym))
                                        }
                                    }
                                }
                                name => {
                                    let evaled_symbol =
                                        Atom::eval(op.clone(), env).context(format!(
                                            "While evaluating symbol {} to see if it is a function",
                                            name
                                        ))?;
                                    match evaled_symbol.as_ref() {
                                        Atom::NativeFunc(f) => f(args.clone()).context(format!(
                                            "While evaluating builtin function bound to {}",
                                            name,
                                        )),
                                        s => Err(eyre!(
                                            "Expected a function or a builtin function, got {}",
                                            s
                                        )),
                                    }
                                }
                            }
                        }
                        _ => Err(eyre!(
                            "Expected a function as first element of evaluated list"
                        )),
                    }
                } else {
                    Err(eyre!("Attempted to evaluate improper list"))
                }
            }
            Atom::NativeFunc(_) => Ok(expr),
        }
    }
}
