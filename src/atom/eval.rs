use std::{collections::VecDeque, rc::Rc};

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use gc::Gc;
use tracing::{debug, info, instrument, warn};

use super::Atom;
use crate::env::Env;

struct StackFrame {
    /// The stack frame corresponding to the parent expression (that is, the one which is waiting for the result of the current expression)
    parent: Box<StackFrame>,
    /// Current environment
    env: Rc<Env>,
    /// Evaluated operator
    evaluated_operation: Gc<Atom>,
    /// Aruments pending evaluation
    pending_arguments: Vec<Gc<Atom>>,
    /// Arguments following evaluation
    evaluated_arguments: Vec<Gc<Atom>>,
    /// Expressions in the function body which are pending execution
    body: Gc<Atom>,
}

impl Atom {
    #[instrument(skip(env))]
    pub fn eval(expr: Gc<Atom>, env: &mut Env) -> Result<Gc<Atom>> {
        debug!("Starting evalutation of {expr}");
        let mut expr_stack = VecDeque::new();
        expr_stack.push_back(expr);
        loop {
            let expr = expr_stack.pop_back();
            if let Some(expr) = expr {
                debug!("Evalutation of {expr}");
                match expr.as_ref() {
                    Atom::Number(_)
                    | Atom::NativeFunc(_)
                    | Atom::Closure(_, _, _)
                    | Atom::String(_) => {
                        debug!("Primitive {expr} evaluates to itself");
                        return Ok(expr.clone());
                    }
                    Atom::Macro(_, _, _) => {
                        warn!("Attempt to evaluate macro {expr}");
                        return Err(eyre!("Attempt to evaluate macro {}", expr));
                    }
                    Atom::Symbol(symbol) => return env.get(symbol),
                    Atom::Pair(car, cdr) => {
                        debug!("Starting pair evaluation for {expr}");
                        if !Atom::is_proper_list(expr.clone()) {
                            warn!("Attempted to evaluate improper list {expr}");
                            return Err(eyre!("Attempted to evaluate improper list\n{}", expr));
                        } else {
                            match car.as_ref() {
                                Atom::Number(_) | Atom::String(_) | Atom::Macro(_, _, _) => {
                                    warn!("Expected a function as first element of evaluated list, got {car}");
                                    return Err(eyre!("Expected a function as first element of evaluated list, got\n{}", car));
                                }
                            }
                        }
                    }
                }
            } else {
                return Err(eyre!("Loop without remaining expr to evaluate"));
            }
        }
    }

    /// Old_Evaluate a single atom.
    #[instrument(skip(env))]
    pub fn old_eval(expr: Gc<Atom>, env: &mut Env) -> Result<Gc<Atom>> {
        match expr.as_ref() {
            Atom::Number(_) | Atom::NativeFunc(_) | Atom::Closure(_, _, _) | Atom::String(_) => {
                debug!("Primitive old_evaluates to itself");
                Ok(expr.clone())
            }
            Atom::Symbol(symbol) => env.get(symbol),
            Atom::Macro(_, _, _) => Err(eyre!("Attempt to old_evaluate macro {}", expr)),
            Atom::Pair(car, cdr) => list_old_evaluation(car, cdr, &expr, env),
        }
    }
}

fn old_eval_elements_in_list(x: Gc<Atom>, env: &mut Env) -> Result<Gc<Atom>> {
    Ok(Gc::new(Atom::Pair(
        Atom::old_eval(x.car()?, env)?,
        if x.cdr()?.is_nil() {
            x.cdr()?
        } else {
            old_eval_elements_in_list(x.cdr()?, env)?
        },
    )))
}

fn list_old_evaluation(
    car: &Gc<Atom>,
    cdr: &Gc<Atom>,
    expr: &Gc<Atom>,
    env: &mut Env,
) -> Result<Gc<Atom>, color_eyre::Report> {
    if !Atom::is_proper_list(expr.clone()) {
        Err(eyre!("Attempted to old_evaluate improper list\n{}", expr))
    } else {
        let op = Atom::old_eval(car.clone(), env).context(format!(
            "While old_evaluating first element of list for function application {:?}",
            car,
        ))?;
        let args = cdr;

        match op.as_ref() {
            Atom::Symbol(symbol) => try_old_evaluate_special_form(symbol, args, env).context(
                format!("While trying to old_evaluate special form {:?}", symbol),
            ),
            Atom::NativeFunc(f) => {
                let old_evaled_args = old_eval_elements_in_list(args.clone(), env)?;
                f(old_evaled_args)
                    .context(format!("While old_evaluating builtin function {:?}", expr))
            }
            Atom::Closure(function_env, original_arg_names, body) => {
                old_eval_closure(function_env, env, original_arg_names, args, body)
                    .context(format!("While old_evaluating closure\n{}", expr))
            }
            Atom::Macro(function_env, original_arg_names, body) => {
                old_eval_macro(function_env, env, original_arg_names, args, body)
                    .context(format!("While old_evaluating macro\n{}", expr))
            }
            a => Err(eyre!(
                "Expected a function as first element of old_evaluated list, got\n{}",
                a
            )),
        }
    }
}

fn old_eval_macro(
    function_env: &Env,
    env: &mut Env,
    original_arg_names: &Gc<Atom>,
    args: &Gc<Atom>,
    body: &Gc<Atom>,
) -> Result<Gc<Atom>, color_eyre::Report> {
    let mut func_env = Env::new(Some(Box::new(function_env.clone())));
    func_env.add_furthest_parent(env.clone());
    let mut arg_names = Gc::new(original_arg_names.as_ref().clone());
    let mut args_working = Gc::new(args.as_ref().clone());
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

                func_env.set(sym.to_string(), args_working.clone());
                args_working = Gc::new(Atom::nil());
                break;
            }
            _ => {
                let arg = args_working.car()?;
                func_env.set(arg_names.car()?.get_symbol_name()?, arg);
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
        let mut body_working = Gc::new(body.as_ref().clone());

        let mut result = Gc::new(Atom::nil());

        while !body_working.is_nil() {
            let to_old_eval = body_working.car()?;
            result = Atom::old_eval(to_old_eval.clone(), &mut func_env)
                .context(format!("While old_evaluating closure\n{}", to_old_eval))?;
            result = Atom::old_eval(result, &mut func_env)?;
            body_working = body_working.cdr()?;
        }

        Ok(result)
    }
}

fn old_eval_closure(
    function_env: &Env,
    env: &mut Env,
    original_arg_names: &Gc<Atom>,
    args: &Gc<Atom>,
    body: &Gc<Atom>,
) -> Result<Gc<Atom>, color_eyre::Report> {
    let mut func_env = Env::new(Some(Box::new(function_env.clone())));
    func_env.add_furthest_parent(env.clone());
    let mut arg_names = Gc::new(original_arg_names.as_ref().clone());
    let mut args_working = Gc::new(args.as_ref().clone());
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
                // old_eval each arg
                fn old_eval_args(x: Gc<Atom>, env: &mut Env) -> Result<Gc<Atom>> {
                    Ok(Gc::new(Atom::Pair(
                        Atom::old_eval(x.car()?, env)?,
                        if x.cdr()?.is_nil() {
                            x.cdr()?
                        } else {
                            old_eval_args(x.cdr()?, env)?
                        },
                    )))
                }
                let old_evaled_args = old_eval_args(args_working.clone(), env)?;

                func_env.set(sym.to_string(), old_evaled_args);
                args_working = Gc::new(Atom::nil());
                break;
            }
            _ => {
                let arg = args_working.car()?;
                let old_evaled_arg = Atom::old_eval(arg, env)?;
                func_env.set(arg_names.car()?.get_symbol_name()?, old_evaled_arg);
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
        let mut body_working = Gc::new(body.as_ref().clone());

        let mut result = Gc::new(Atom::nil());

        while !body_working.is_nil() {
            let to_old_eval = body_working.car()?;
            result = Atom::old_eval(to_old_eval.clone(), &mut func_env)
                .context(format!("While old_evaluating closure\n{}", to_old_eval))?;
            body_working = body_working.cdr()?;
        }

        Ok(result)
    }
}

fn try_old_evaluate_special_form(
    symbol: &str,
    args: &Gc<Atom>,
    env: &mut Env,
) -> Result<Gc<Atom>, color_eyre::Report> {
    match symbol {
        "quote" => old_eval_special_form_quote(args).context(format!(
            "While trying to old_evaluate special form quote with args\n{}",
            args
        )),
        "define" => old_eval_special_form_define(args, env).context(format!(
            "While trying to old_evaluate special form define with args\n{}",
            args
        )),
        "defmacro" => old_eval_special_form_defmacro(args, env).context(format!(
            "While trying to old_evaluate special form defmacro with args\n{}",
            args
        )),
        "lambda" => old_eval_special_form_lambda(args, env).context(format!(
            "While trying to old_evaluate special form lambda with args\n{}",
            args
        )),
        "if" => old_eval_special_form_if(args, env).context(format!(
            "While trying to old_evaluate special form if with args\n{}",
            args
        )),
        "apply" => old_eval_special_form_apply(args, env).context(format!(
            "While trying to old_evaluate special form apply with args\n{}",
            args
        )),
        name => Err(eyre!(
            "Expected function, builtin function or special form, but got {}, which is a symbol",
            name
        )),
    }
}

fn old_eval_special_form_apply(
    args: &Gc<Atom>,
    env: &mut Env,
) -> Result<Gc<Atom>, color_eyre::Report> {
    if args.is_nil() || args.cdr()?.is_nil() || !args.cdr()?.cdr()?.is_nil() {
        Err(eyre!(
            "Special form apply expected exactly two arguments, got {}",
            args
        ))
    } else {
        let func = Atom::old_eval(args.car()?, env)?;
        let args = Atom::old_eval(args.cdr()?.car()?, env)?;
        if !Atom::is_proper_list(args.clone()) {
            Err(eyre!("Expected second argument to apply to be a proper list, but got {}, which is invalid", args))
        } else {
            let to_old_eval = Gc::new(Atom::Pair(func, quote_elements_in_list(args)?));
            Atom::old_eval(to_old_eval, env)
        }
    }
}

fn quote_elements_in_list(x: Gc<Atom>) -> Result<Gc<Atom>> {
    Ok(Gc::new(Atom::Pair(
        Gc::new(Atom::Pair(
            Gc::new(Atom::symbol("quote")),
            Gc::new(Atom::Pair(x.car()?, Gc::new(Atom::symbol("nil")))),
        )),
        if x.cdr()?.is_nil() {
            x.cdr()?
        } else {
            quote_elements_in_list(x.cdr()?)?
        },
    )))
}

fn old_eval_special_form_if(
    args: &Gc<Atom>,
    env: &mut Env,
) -> Result<Gc<Atom>, color_eyre::Report> {
    if args.is_nil()
        || args.cdr()?.is_nil()
        || args.cdr()?.cdr()?.is_nil()
        || !args.cdr()?.cdr()?.cdr()?.is_nil()
    {
        Err(eyre!(
            "Special form if takes exactly 3 arguments, but got {}, which is invalid",
            args
        ))
    } else {
        let result = Atom::old_eval(args.car()?, env)?;
        if result.as_bool() {
            Atom::old_eval(args.cdr()?.car()?, env)
        } else {
            Atom::old_eval(args.cdr()?.cdr()?.car()?, env)
        }
    }
}

fn old_eval_special_form_lambda(
    args: &Gc<Atom>,
    env: &mut Env,
) -> Result<Gc<Atom>, color_eyre::Report> {
    if args.is_nil() || args.cdr()?.is_nil() {
        Err(eyre!(
            "LAMBDA has the form (lambda (arg ...) (body) ...), but got {}, which is invalid",
            args
        ))
    } else {
        Atom::closure(env.clone(), args.car()?, args.cdr()?)
    }
}

fn old_eval_special_form_defmacro(
    args: &Gc<Atom>,
    env: &mut Env,
) -> Result<Gc<Atom>, color_eyre::Report> {
    if args.is_nil() || args.cdr()?.is_nil() || !matches!(args.as_ref(), Atom::Pair(_, _)) {
        Err(eyre!("DEFMACRO has the form (DEFMACRO (name arg ...) body ...), but got {}, which is invalid", args))
    } else {
        let name = args.car()?.car()?;
        match name.as_ref() {
            Atom::Symbol(sym) => {
                let (macro_env, args, body) =
                    Atom::validate_closure_form(env.clone(), args.car()?.cdr()?, args.cdr()?)?;
                let makro = Gc::new(Atom::Macro(macro_env, args, body));
                env.set(sym.to_string(), makro);
                Ok(name)
            }
            a => Err(eyre!("Expected name to be a symbol, got {}", a)),
        }
    }
}

fn old_eval_special_form_define(
    args: &Gc<Atom>,
    env: &mut Env,
) -> Result<Gc<Atom>, color_eyre::Report> {
    // exactly two arguments
    if args.is_nil() || args.cdr()?.is_nil() {
        Err(eyre!(
            "DEFINE has either the form (DEFINE name value) or (DEFINE (name arg ...) body ...), but got {}, which is invalid",
            &args
        ))
    } else {
        let sym = args.car()?;
        match sym.as_ref() {
            Atom::Pair(car, cdr) => {
                let result = Atom::closure(env.clone(), cdr.clone(), args.cdr()?)?;
                match car.as_ref() {
                    Atom::Symbol(symbol) => {
                        let symbol = symbol.to_string();

                        // set closure name in environment.
                        let result = Atom::closure_add_env_binding(result.clone(), symbol.clone(), result)?;

                        env.set(symbol, result);
                        Ok(car.clone())
                    }
                    _ => {
                        Err(eyre!("Found define form (DEFINE (name arg ...) body ...), but name was not a symbol"))
                    }
                }
            }
            Atom::Symbol(symbol) => {
                let value = Atom::old_eval(args.cdr()?.car()?, env)
                    .context("While old_evaluating VALUE argument for DEFINE")?;
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

fn old_eval_special_form_quote(args: &Gc<Atom>) -> Result<Gc<Atom>, color_eyre::Report> {
    // exactly one argument
    if args.is_nil() || !args.cdr().context("expected args to be a list")?.is_nil() {
        Err(eyre!("QUOTE takes exactly one argument, got {}", &args))
    } else {
        args.car()
    }
}
