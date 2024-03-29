use std::rc::Rc;

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use tracing::{debug, instrument};

use super::Atom;
use crate::env::Env;

impl Atom {
    /// Evaluate a single atom.
    #[instrument(skip(env))]
    pub fn eval(expr: Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>> {
        match expr.as_ref() {
            Atom::Number(_) | Atom::NativeFunc(_) | Atom::Closure(_, _, _) | Atom::String(_) => {
                debug!("Primitive evaluates to itself");
                Ok(expr.clone())
            }
            Atom::Symbol(symbol) => env.get(symbol),
            Atom::Macro(_, _, _) => Err(eyre!("Attempt to evaluate macro {}", expr)),
            Atom::Pair(car, cdr) => list_evaluation(car, cdr, &expr, env),
        }
    }
}

fn eval_elements_in_list(x: &Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>> {
    Ok(Rc::new(Atom::Pair(
        Atom::eval(x.car(), env)?,
        if x.cdr().is_nil() {
            x.cdr()
        } else {
            eval_elements_in_list(&x.cdr(), env)?
        },
    )))
}

fn list_evaluation(
    car: &Rc<Atom>,
    cdr: &Rc<Atom>,
    expr: &Rc<Atom>,
    env: &mut Env,
) -> Result<Rc<Atom>, color_eyre::Report> {
    if !Atom::is_proper_list(expr.clone()) {
        return Err(eyre!("Attempted to evaluate improper list\n{}", expr));
    }
    let op = Atom::eval(car.clone(), env).context(format!(
        "While evaluating first element of list for function application {:?}",
        car,
    ))?;
    let args = cdr;

    match &op.as_ref() {
        Atom::Symbol(symbol) => try_evaluate_special_form(symbol, args, env).context(format!(
            "While trying to evaluate special form {:?}",
            symbol
        )),
        Atom::NativeFunc(f) => {
            let evaled_args = eval_elements_in_list(&args.clone(), env)?;
            f(evaled_args).context(format!("While evaluating builtin function {:?}", expr))
        }
        Atom::Closure(function_env, original_arg_names, body) => {
            eval_closure(function_env, env, original_arg_names, args, body)
                .context(format!("While evaluating closure\n{}", expr))
        }
        Atom::Macro(function_env, original_arg_names, body) => {
            eval_macro(function_env, env, original_arg_names, args, body)
                .context(format!("While evaluating macro\n{}", expr))
        }
        a => Err(eyre!(
            "Expected a function as first element of evaluated list, got\n{}",
            a
        )),
    }
}

fn eval_macro(
    function_env: &Env,
    env: &mut Env,
    original_arg_names: &Rc<Atom>,
    args: &Rc<Atom>,
    body: &Rc<Atom>,
) -> Result<Rc<Atom>, color_eyre::Report> {
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

        if let Atom::Symbol(sym) = arg_names.as_ref() {
            // final argument for variadic functions

            func_env.set(sym.to_string(), args_working.clone());
            args_working = Rc::new(Atom::nil());
            break;
        } else {
            let arg = args_working.car();
            func_env.set(arg_names.car().get_symbol_name()?, arg);
            arg_names = arg_names.cdr();
            args_working = args_working.cdr();
        }
    }
    if args_working.is_nil() {
        let mut body_working = Rc::new(body.as_ref().clone());

        let mut result = Rc::new(Atom::nil());

        while !body_working.is_nil() {
            let to_eval = body_working.car();
            result = Atom::eval(to_eval.clone(), &mut func_env)
                .context(format!("While evaluating closure\n{}", to_eval))?;
            result = Atom::eval(result, &mut func_env)?;
            body_working = body_working.cdr();
        }

        Ok(result)
    } else {
        Err(eyre!(
            "Too many arguments, expected {} but got {}",
            original_arg_names,
            args
        ))
    }
}

fn eval_closure(
    function_env: &Env,
    env: &mut Env,
    original_arg_names: &Rc<Atom>,
    args: &Rc<Atom>,
    body: &Rc<Atom>,
) -> Result<Rc<Atom>, color_eyre::Report> {
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

        if let Atom::Symbol(sym) = arg_names.as_ref() {
            // final argument for variadic functions
            // eval each arg
            fn eval_args(x: &Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>> {
                Ok(Rc::new(Atom::Pair(
                    Atom::eval(x.car(), env)?,
                    if x.cdr().is_nil() {
                        x.cdr()
                    } else {
                        eval_args(&x.cdr(), env)?
                    },
                )))
            }
            let evaled_args = eval_args(&args_working, env)?;

            func_env.set(sym.to_string(), evaled_args);
            args_working = Rc::new(Atom::nil());
            break;
        } else {
            let arg = args_working.car();
            let evaled_arg = Atom::eval(arg, env)?;
            func_env.set(arg_names.car().get_symbol_name()?, evaled_arg);
            arg_names = arg_names.cdr();
            args_working = args_working.cdr();
        }
    }
    if args_working.is_nil() {
        let mut body_working = Rc::new(body.as_ref().clone());

        let mut result = Rc::new(Atom::nil());

        while !body_working.is_nil() {
            let to_eval = body_working.car();
            result = Atom::eval(to_eval.clone(), &mut func_env)
                .context(format!("While evaluating closure\n{}", to_eval))?;
            body_working = body_working.cdr();
        }

        Ok(result)
    } else {
        Err(eyre!(
            "Too many arguments, expected {} but got {}",
            original_arg_names,
            args
        ))
    }
}

fn try_evaluate_special_form(
    symbol: &str,
    args: &Rc<Atom>,
    env: &mut Env,
) -> Result<Rc<Atom>, color_eyre::Report> {
    match symbol {
        "quote" => eval_special_form_quote(args).context(format!(
            "While trying to evaluate special form quote with args\n{}",
            args
        )),
        "define" => eval_special_form_define(args, env).context(format!(
            "While trying to evaluate special form define with args\n{}",
            args
        )),
        "defmacro" => eval_special_form_defmacro(args, env).context(format!(
            "While trying to evaluate special form defmacro with args\n{}",
            args
        )),
        "lambda" => eval_special_form_lambda(args, env).context(format!(
            "While trying to evaluate special form lambda with args\n{}",
            args
        )),
        "if" => eval_special_form_if(args, env).context(format!(
            "While trying to evaluate special form if with args\n{}",
            args
        )),
        "apply" => eval_special_form_apply(args, env).context(format!(
            "While trying to evaluate special form apply with args\n{}",
            args
        )),
        name => Err(eyre!(
            "Expected function, builtin function or special form, but got {}, which is a symbol",
            name
        )),
    }
}

fn eval_special_form_apply(args: &Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>, color_eyre::Report> {
    if args.is_nil() || args.cdr().is_nil() || !args.cdr().cdr().is_nil() {
        Err(eyre!(
            "Special form apply expected exactly two arguments, got {}",
            args
        ))
    } else {
        let func = Atom::eval(args.car(), env)?;
        let args = Atom::eval(args.cdr().car(), env)?;
        if Atom::is_proper_list(args.clone()) {
            let to_eval = Rc::new(Atom::Pair(func, quote_elements_in_list(&args)?));
            Atom::eval(to_eval, env)
        } else {
            Err(eyre!("Expected second argument to apply to be a proper list, but got {}, which is invalid", args))
        }
    }
}

fn quote_elements_in_list(x: &Rc<Atom>) -> Result<Rc<Atom>> {
    Ok(Rc::new(Atom::Pair(
        Rc::new(Atom::Pair(
            Rc::new(Atom::symbol("quote")),
            Rc::new(Atom::Pair(x.car(), Rc::new(Atom::symbol("nil")))),
        )),
        if x.cdr().is_nil() {
            x.cdr()
        } else {
            quote_elements_in_list(&x.cdr())?
        },
    )))
}

fn eval_special_form_if(args: &Rc<Atom>, env: &mut Env) -> Result<Rc<Atom>, color_eyre::Report> {
    if args.is_nil()
        || args.cdr().is_nil()
        || args.cdr().cdr().is_nil()
        || !args.cdr().cdr().cdr().is_nil()
    {
        Err(eyre!(
            "Special form if takes exactly 3 arguments, but got {}, which is invalid",
            args
        ))
    } else {
        let result = Atom::eval(args.car(), env)?;
        if result.as_bool() {
            Atom::eval(args.cdr().car(), env)
        } else {
            Atom::eval(args.cdr().cdr().car(), env)
        }
    }
}

fn eval_special_form_lambda(
    args: &Rc<Atom>,
    env: &mut Env,
) -> Result<Rc<Atom>, color_eyre::Report> {
    if args.is_nil() || args.cdr().is_nil() {
        Err(eyre!(
            "LAMBDA has the form (lambda (arg ...) (body) ...), but got {}, which is invalid",
            args
        ))
    } else {
        Atom::closure(env.clone(), args.car(), args.cdr())
    }
}

fn eval_special_form_defmacro(
    args: &Rc<Atom>,
    env: &mut Env,
) -> Result<Rc<Atom>, color_eyre::Report> {
    if args.is_nil() || args.cdr().is_nil() || !matches!(args.as_ref(), Atom::Pair(_, _)) {
        Err(eyre!("DEFMACRO has the form (DEFMACRO (name arg ...) body ...), but got {}, which is invalid", args))
    } else {
        let name = args.car().car();
        match name.as_ref() {
            Atom::Symbol(sym) => {
                let (macro_env, args, body) =
                    Atom::validate_closure_form(env.clone(), args.car().cdr(), args.cdr())?;
                let makro = Rc::new(Atom::Macro(macro_env, args, body));
                env.set(sym.to_string(), makro);
                Ok(name)
            }
            a => Err(eyre!("Expected name to be a symbol, got {}", a)),
        }
    }
}

fn eval_special_form_define(
    args: &Rc<Atom>,
    env: &mut Env,
) -> Result<Rc<Atom>, color_eyre::Report> {
    // exactly two arguments
    if args.is_nil() || args.cdr().is_nil() {
        Err(eyre!(
            "DEFINE has either the form (DEFINE name value) or (DEFINE (name arg ...) body ...), but got {}, which is invalid",
            &args
        ))
    } else {
        let sym = args.car();
        match sym.as_ref() {
            Atom::Pair(car, cdr) => {
                let result = Atom::closure(env.clone(), cdr.clone(), args.cdr())?;
                match car.as_ref() {
                    Atom::Symbol(symbol) => {
                        let symbol = symbol.to_string();

                        // set closure name in environment.
                        let result = Atom::closure_add_env_binding(&result.clone(), symbol.clone(), result)?;

                        env.set(symbol, result);
                        Ok(car.clone())
                    }
                    _ => {
                        Err(eyre!("Found define form (DEFINE (name arg ...) body ...), but name was not a symbol"))
                    }
                }
            }
            Atom::Symbol(symbol) => {
                let value = Atom::eval(args.cdr().car(), env)
                    .context("While evaluating VALUE argument for DEFINE")?;
                env.set(symbol.to_string(), value);
                Ok(sym)
            }
            _ => Err(eyre!(
                "Expected a symbol as first argument to define, got {}",
                sym
            )),
        }
    }
}

fn eval_special_form_quote(args: &Rc<Atom>) -> Result<Rc<Atom>, color_eyre::Report> {
    // exactly one argument
    if args.is_nil() || !args.cdr().is_nil() {
        Err(eyre!("QUOTE takes exactly one argument, got {}", &args))
    } else {
        Ok(args.car())
    }
}
