use std::rc::Rc;

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};

use super::Atom;
use crate::env::Env;

impl Atom {
    pub fn eval(expr: Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>> {
        let result = match expr.as_ref() {
            Atom::Number(_) => Ok(expr.clone()),
            Atom::NativeFunc(_) => Ok(expr.clone()),
            Atom::Symbol(symbol) => Ok(env
                .get(symbol)
                .ok_or_else(|| eyre!("Symbol {} is not bound to any value", symbol))?),
            Atom::Pair(car, cdr) => {
                if Atom::is_proper_list(expr.clone()) {
                    let op = Atom::eval(car.clone(), env).context(format!(
                        "While evaluating first element of list for function application {}",
                        car,
                    ))?;
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
                                    {
                                        Err(eyre!(
                                            "DEFINE has either the form (DEFINE name value) or (DEFINE (name args ...) body ...), but got {}, which is invalid",
                                            &args
                                        ))
                                    } else {
                                        let sym = args.car()?;
                                        match sym.as_ref() {
                                            Atom::Pair(car, cdr) => {
                                                let result = Atom::closure(env.clone(), cdr.clone(), args.cdr()?)?;
                                                match car.as_ref() {
                                                    Atom::Symbol(symbol) => {
                                                        env.set(symbol.to_string(), result);
                                                        Ok(car.clone())
                                                    }
                                                    _ => {
                                                        Err(eyre!("Found define form (DEFINE (name args ...) body ...), but name was not a symbol"))
                                                    }
                                                }
                                            }
                                            Atom::Symbol(symbol) => {
                                                let value = Atom::eval(args.cdr()?.car()?, env).context("While evaluating VALUE argument for DEFINE")?;
                                                env.set(symbol.to_string(), value);
                                                Ok(sym)
                                            },
                                            _ => Err(eyre!("Expected a symbol as first argument to define, got {}", sym))
                                        }
                                    }
                                }
                                "lambda" => {
                                    if args.is_nil() || args.cdr()?.is_nil() {
                                        Err(eyre!("LAMBDA has the form (lambda (args ...) (body) ...), but got {args}, which is invalid"))
                                    } else {
                                        Atom::closure(env.clone(), args.car()?, args.cdr()?)
                                    }
                                }
                                "if" => {
                                    if args.is_nil() || args.cdr()?.is_nil() || args.cdr()?.cdr()?.is_nil() || !args.cdr()?.cdr()?.cdr()?.is_nil() {
                                        Err(eyre!("Special form if takes exactly 3 arguments, but got {}, which is invalid", args))
                                    } else {
                                        let result = Atom::eval(args.car()?, env)?;
                                        if result.as_bool() {
                                            Atom::eval(args.cdr()?.car()?, env)
                                        } else {
                                            Atom::eval(args.cdr()?.cdr()?.car()?, env)
                                        }
                                    }
                                }
                                name => {
                                    Err(eyre!("Expected function, builtin function or special form, but got {}, which is a symbol", name))
                                }
                            }
                        }
                        Atom::NativeFunc(f) => {
                            fn eval_args(x: Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>> {
                                Ok(Rc::new(Atom::Pair(
                                    Atom::eval(x.car()?, env)?,
                                    if x.cdr()?.is_nil() {
                                        x.cdr()?
                                    } else {
                                        eval_args(x.cdr()?, env)?
                                    },
                                )))
                            }
                            let evaled_args = eval_args(args.clone(), env)?;
                            f(evaled_args, env).context("While evaluating builtin function")
                        }
                        Atom::Closure(function_env, original_arg_names, body) => {
                            let mut func_env = Env::new(Some(Box::new(function_env.clone())));
                            func_env.add_furthest_parent(env.clone());
                            let mut arg_names = Rc::new(original_arg_names.as_ref().clone());
                            let mut args_working = Rc::new(args.as_ref().clone());
                            while !arg_names.is_nil() {
                                if args_working.is_nil() {
                                    return Err(eyre!(
                                        "Too few arguments, expected {}, but got {}",
                                        arg_names,
                                        args
                                    ));
                                }

                                match arg_names.as_ref() {
                                    Atom::Symbol(sym) => {
                                        // final argument for variadic functions
                                        // eval each arg
                                        fn eval_args(
                                            x: Rc<Atom>,
                                            env: &mut Env,
                                        ) -> Result<Rc<Atom>>
                                        {
                                            Ok(Rc::new(Atom::Pair(
                                                Atom::eval(x.car()?, env)?,
                                                if x.cdr()?.is_nil() {
                                                    x.cdr()?
                                                } else {
                                                    eval_args(x.cdr()?, env)?
                                                },
                                            )))
                                        }
                                        let evaled_args = eval_args(args_working.clone(), env)?;

                                        func_env.set(sym.to_string(), evaled_args);
                                        args_working = Rc::new(Atom::nil());
                                        break;
                                    }
                                    _ => {
                                        let arg = args_working.car()?;
                                        let evaled_arg = Atom::eval(arg, env)?;
                                        func_env
                                            .set(arg_names.car()?.get_symbol_name()?, evaled_arg);
                                        arg_names = arg_names.cdr()?;
                                        args_working = args_working.cdr()?;
                                    }
                                }
                            }

                            if !args_working.is_nil() {
                                Err(eyre!(
                                    "Too many arguments, expected {} but got {}",
                                    original_arg_names,
                                    args
                                ))
                            } else {
                                let mut body_working = Rc::new(body.as_ref().clone());

                                let mut result = Rc::new(Atom::nil());

                                while !body_working.is_nil() {
                                    let to_eval = body.car()?;
                                    result = Atom::eval(to_eval.clone(), &mut func_env)
                                        .context(format!("While evaluating closure {}", to_eval))?;
                                    body_working = body_working.cdr()?;
                                }

                                Ok(result)
                            }
                        }
                        a => Err(eyre!(
                            "Expected a function as first element of evaluated list, got {}",
                            a
                        )),
                    }
                } else {
                    Err(eyre!("Attempted to evaluate improper list"))
                }
            }
            Atom::Closure(_, _, _) => Err(eyre!("Attempt to evaluate closure {}", expr)),
        };
        result
    }
}
