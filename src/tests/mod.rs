use std::rc::Rc;

use chumsky::Parser;

use crate::{atom::Atom, env::Env, parsing::parser};

fn run_code(mut src: &str) -> Rc<Atom> {
    let mut env = Env::default();
    src = src.trim();
    let atoms = parser()
        .parse(src)
        .expect("The given source code should have no errors");
    let mut final_result = Rc::new(Atom::nil());
    for atom in atoms {
        let atom = Rc::new(atom);
        let result = Atom::eval(atom.clone(), &mut env);

        match result {
            Ok(result) => {
                final_result = result.clone();
                println!("{}", atom);
                println!("=> {}", result);
            }
            Err(e) => {
                panic!("{}\n!! {:?}", atom, e);
            }
        }
    }
    final_result
}

fn helper(src: &str, expected: &str) {
    let result = run_code(src);
    let expected = run_code(expected);
    assert_eq!(result, expected);
}

// //// //// //// // BASIC TESTS // //// //// //// //

#[test]
fn can_run_empty_string() {
    let src = "";
    let expected = Atom::nil();
    assert_eq!(run_code(src).as_ref().clone(), expected);
}

#[test]
fn nil_is_nil() {
    let src = "nil";
    let expected = Atom::nil();
    assert_eq!(run_code(src).as_ref().clone(), expected);
}

#[test]
fn empty_list_is_nil() {
    let src = "()";
    let expected = Atom::nil();
    assert_eq!(run_code(src).as_ref().clone(), expected);
}

#[test]
fn t_is_t() {
    let src = "t";
    let expected = Atom::t();
    assert_eq!(run_code(src).as_ref().clone(), expected);
}

#[test]
fn x_is_x() {
    fn x(x: &str) {
        helper(x, x);
    }
    x("define");
    x("defmacro");
    x("lambda");
    x("if");
    x("quote");
    x("apply");
}

#[test]
fn builtins_exist() {
    fn exists(x: &str) {
        run_code(x);
    }

    exists("into-pretty-string");
    exists("into-string");
    exists("print");
    exists("println");
    exists("pair?");
    exists("symbol?");
    exists("string?");
    exists("string-length");
    exists("car");
    exists("cdr");
    exists("cons");
    exists("+");
    exists("-");
    exists("*");
    exists("/");
    exists("%");
    exists("=");
    exists("<");
    exists(">");
    exists(">=");
    exists("<=");
}

// //// //// //// // INTEGRATION TESTS // //// //// //// //

#[test]
fn can_load_standard_library() {
    let src = include_str!("../../lib/lib.lisp");
    run_code(src);
}
