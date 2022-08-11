use std::rc::Rc;

use chumsky::Parser;

use crate::{atom::Atom, env::Env, parsing::parser};

fn parse_has_error(mut src: &str) {
    src = src.trim();
    println!("{}", &src);
    assert!(parser().parse(src).is_err());
}

fn parse(mut src: &str) -> Vec<Atom> {
    src = src.trim();
    println!("{}", &src);
    parser()
        .parse(src)
        .expect("The given source code should have no errors")
}

fn parse_one(src: &str) -> Atom {
    let atoms = parse(src);
    let first = atoms.first().expect("Expected exactly one atom").clone();
    assert_eq!(atoms.len(), 1, "Expected exactly one atom");
    first
}

// converts a Vec<Atom> into a corresponding lisp cons list
fn create_list(x: &[Atom]) -> Atom {
    match x.first().cloned() {
        Some(first) => Atom::cons(first, create_list(&x[1..])),
        None => Atom::nil(),
    }
}

fn run_code(src: &str) -> Rc<Atom> {
    let mut env = Env::default();
    let atoms = parse(src);
    let mut final_result = Rc::new(Atom::nil());
    for atom in atoms {
        let atom = Rc::new(atom);
        let result = Atom::eval(atom.clone(), &mut env);

        match result {
            Ok(result) => {
                final_result = result.clone();
                print!("{}", atom);
                println!(" => {}", result);
            }
            Err(e) => {
                panic!("{} !! {:?}", atom, e);
            }
        }
    }
    final_result
}

fn run_has_error(src: &str) {
    let mut env = Env::default();
    let atoms = parse(src);
    for atom in atoms {
        let atom = Rc::new(atom);
        let result = Atom::eval(atom.clone(), &mut env);

        match result {
            Ok(result) => {
                panic!("{}\n => {}", atom, result);
            }
            Err(e) => {
                println!("{} !! {:?}", atom, e);
            }
        }
    }
}

fn helper(src: &str, expected: &str) {
    print!("result: ");
    let result = run_code(src);
    print!("expected: ");
    let expected = run_code(expected);
    assert_eq!(result, expected);
    println!();
}

fn x(x: &str) {
    helper(x, x);
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
    assert_eq!(parse_one("()"), Atom::nil());
    assert_eq!(parse_one("(  )"), Atom::nil());
}

#[test]
fn t_is_t() {
    let src = "t";
    let expected = Atom::t();
    assert_eq!(run_code(src).as_ref().clone(), expected);
}

#[test]
fn x_is_x() {
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

// //// //// //// // BUILTIN TESTS // //// //// //// //

#[test]
fn smaller_equals() {
    helper("(<= 1 2)", "t");
    helper("(<= 2 2)", "t");
    helper("(<= 3 2)", "nil");
    helper("(<= -3 -2)", "t");
    helper("(<= -2 -2)", "t");
    helper("(<= -1 -2)", "nil");
}

#[test]
fn bigger_equals() {
    helper("(>= 1 2)", "nil");
    helper("(>= 2 2)", "t");
    helper("(>= 3 2)", "t");
    helper("(>= -3 -2)", "nil");
    helper("(>= -2 -2)", "t");
    helper("(>= -1 -2)", "t");
}

#[test]
fn bigger() {
    helper("(> 1 2)", "nil");
    helper("(> 3 2)", "t");
    helper("(> -3 -2)", "nil");
    helper("(> -1 -2)", "t");
}

#[test]
fn smaller() {
    helper("(<= 1 2)", "t");
    helper("(<= 3 2)", "nil");
    helper("(<= -3 -2)", "t");
    helper("(<= -1 -2)", "nil");
}

#[test]
fn equal() {
    helper("(= 1 1)", "t");
    helper("(= 1 0)", "nil");
    helper("(= \"hello\" \"hello\")", "t");
    helper("(= \"hello\" \"world\")", "nil");
}

#[test]
fn modulo() {
    helper("(% 6 3)", "0");
    helper("(% 6 4)", "2");
}

#[test]
fn division() {
    helper("(/ 4 2)", "2");
    helper("(/ 5 2)", "2.5");
    helper("(/ 5.1 2.5)", "2.04");
    helper("(/ -4 2)", "-2");
    helper("(/ 4 -2)", "-2");
    helper("(/ -4 -2)", "2");
}

#[test]
fn multiplication() {
    helper("(* 4 2)", "8");
    helper("(* 5 2)", "10");
    helper("(* 5.1 2.5)", "12.75");
    helper("(* -4 2)", "-8");
    helper("(* 4 -2)", "-8");
    helper("(* -4 -2)", "8");
}

#[test]
fn substraction() {
    helper("(- 4 2)", "2");
    helper("(- 5 2)", "3");
    helper("(- 5.3 2.4)", "2.9");
    helper("(- -4 2)", "-6");
    helper("(- 4 -2)", "6");
    helper("(- -4 -2)", "-2");
}

#[test]
fn addition() {
    helper("(+ 4 2)", "6");
    helper("(+ 5 2)", "7");
    helper("(+ 2.4 2.1)", "4.5");
    helper("(+ -4 2)", "-2");
    helper("(+ 4 -2)", "2");
    helper("(+ -4 -2)", "-6");
}

#[test]
fn cons() {
    helper("(cons 1 2)", "'(1 . 2)");
    helper("(cons 1 (cons 2 3))", "'(1 2 . 3)");
    helper("(cons 1 (cons 2 (cons 3 nil)))", "'(1 2 3)");
}

#[test]
fn cdr() {
    helper("(cdr nil)", "nil");
    helper("(cdr t)", "t");
    helper("(cdr 1)", "1");
    helper("(cdr 'test)", "'test");
    helper("(cdr '(1 2 3))", "'(2 3)");
    helper("(cdr '(1))", "'nil");
    helper("(cdr '(1 (2 3) 4 5))", "'((2 3) 4 5)");
    helper("(cdr '(1 (2 3) (4 5)))", "'((2 3) (4 5))");
    helper("(cdr '(1 (4 5)))", "'((4 5))");
}

#[test]
fn car() {
    helper("(car nil)", "nil");
    helper("(car t)", "t");
    helper("(car 1)", "1");
    helper("(car 'test)", "'test");
    helper("(car '(1 2 3))", "1");
    helper("(car '(1))", "'1");
    helper("(car '((1)))", "'(1)");
    helper("(car '(1 (2 3) 4 5))", "1");
    helper("(car '((1 2 3) 4 5))", "'(1 2 3)");
}

#[test]
fn string_length() {
    helper("(string-length \"\")", "0");
    helper("(string-length \"abc\")", "3");
    helper("(string-length \"👍\")", "1");
}

#[test]
fn is_string() {
    helper("(string? \"Hello World!\")", "t");
    helper("(string? 123.55)", "nil");
    helper("(string? nil)", "nil");
    helper("(string? t)", "nil");
    helper("(string? =)", "nil");
}

#[test]
fn is_symbol() {
    helper("(symbol? t)", "t");
    helper("(symbol? nil)", "t");
    helper("(symbol? 'arbitrary-symbol)", "t");
    helper("(symbol? \"Hello World!\")", "nil");
    helper("(symbol? 123.55)", "nil");
    helper("(symbol? =)", "nil");
}

#[test]
fn is_pair() {
    helper("(pair? (cons 1 2))", "t");
    helper("(pair? (cons 1 (cons 2 3)))", "t");
    helper("(pair? '(1 2 3))", "t");
    helper("(pair? '(1 2 . 3))", "t");
    helper("(pair? '(1 (2 . 3)))", "t");

    helper("(pair? '\"Hello world!\")", "nil");
    helper("(pair? 123)", "nil");
    helper("(pair? =)", "nil");
}

#[test]
fn into_string() {
    helper("(into-string \"string\")", r#""\"string\"""#);
    helper("(into-string 123.4)", r#""123.4""#);
    helper("(into-string t)", r#""t""#);
    helper("(into-string nil)", r#""nil""#);
    helper("(into-string 'arbitrary-symbol)", r#""arbitrary-symbol""#);
    helper("(into-string =)", r##""#<BUILTIN>""##);
    helper("(into-string '(1 2 3))", r##""(1 2 3)""##);
    helper("(into-string '(1 (2 3)))", r##""(1 (2 3))""##);
}

// into-pretty-string is not tested, because it's behaviour may change more often, and is less likely to influence program behaviour
// print and println are not tested, because the side effects are difficult to test

// //// //// //// // MAKE-A-LISP TESTS // //// //// //// //

fn run(src: &str) -> Atom {
    run_code(src).as_ref().clone()
}

#[test]
fn read_numbers() {
    assert_eq!(run("1"), Atom::integer(1));
    assert_eq!(run("7"), Atom::integer(7));
    assert_eq!(run("   7"), Atom::integer(7));
    assert_eq!(run("-123"), Atom::integer(-123));
}

#[test]
fn read_symbol() {
    assert_eq!(parse_one("+"), Atom::symbol("+"));
    assert_eq!(parse_one("abc"), Atom::symbol("abc"));
    assert_eq!(parse_one("   abc"), Atom::symbol("abc"));
    assert_eq!(parse_one("abc5"), Atom::symbol("abc5"));
    assert_eq!(parse_one("abc-def"), Atom::symbol("abc-def"));
}

#[test]
fn read_symbol_starting_with_dash() {
    assert_eq!(parse_one("-"), Atom::symbol("-"));
    assert_eq!(parse_one("-abc"), Atom::symbol("-abc"));
    assert_eq!(parse_one("->>"), Atom::symbol("->>"));
}

#[test]
fn read_list() {
    assert_eq!(
        parse_one("(+ 1 2)"),
        Atom::Pair(
            Rc::new(Atom::symbol("+")),
            Rc::new(Atom::Pair(
                Rc::new(Atom::integer(1)),
                Rc::new(Atom::Pair(Rc::new(Atom::integer(2)), Rc::new(Atom::nil())))
            ))
        )
    );

    assert_eq!(
        parse_one("(+ 1 2)"),
        create_list(&[Atom::symbol("+"), Atom::integer(1), Atom::integer(2)])
    );

    assert_eq!(parse_one("(nil)"), create_list(&[Atom::nil()]));

    assert_eq!(
        parse_one("((3 4))"),
        create_list(&[create_list(&[Atom::integer(3), Atom::integer(4)])])
    );

    assert_eq!(
        parse_one("(+ 1 (+ 2 3))"),
        create_list(&[
            Atom::symbol("+"),
            Atom::integer(1),
            create_list(&[Atom::symbol("+"), Atom::integer(2), Atom::integer(3)])
        ])
    );

    assert_eq!(
        parse_one("  ( +   1   (+   2 3   )   )  "),
        create_list(&[
            Atom::symbol("+"),
            Atom::integer(1),
            create_list(&[Atom::symbol("+"), Atom::integer(2), Atom::integer(3)])
        ])
    );

    assert_eq!(
        parse_one("(* 1 2)"),
        create_list(&[Atom::symbol("*"), Atom::integer(1), Atom::integer(2)])
    );

    assert_eq!(
        parse_one("(** 1 2)"),
        create_list(&[Atom::symbol("**"), Atom::integer(1), Atom::integer(2)])
    );

    assert_eq!(
        parse_one("(* -3 6)"),
        create_list(&[Atom::symbol("*"), Atom::integer(-3), Atom::integer(6)])
    );

    assert_eq!(
        parse_one("(() ())"),
        create_list(&[Atom::nil(), Atom::nil()])
    );
}

#[test]
fn read_nil_true_false() {
    assert_eq!(parse_one("nil"), Atom::symbol("nil"));
    assert_eq!(parse_one("true"), Atom::symbol("true"));
    assert_eq!(parse_one("false"), Atom::symbol("false"));
}

#[test]
fn read_string() {
    assert_eq!(parse_one("\"abc\""), Atom::string("abc"));
    assert_eq!(parse_one("   \"abc\""), Atom::string("abc"));
    assert_eq!(
        parse_one("\"abc (with parens)\""),
        Atom::string("abc (with parens)")
    );
    assert_eq!(parse_one(r#""abc\"def""#), Atom::string("abc\"def"));
    assert_eq!(parse_one("\"\""), Atom::string(""));
    assert_eq!(parse_one(r#""\\""#), Atom::string(r#"\"#));
    assert_eq!(
        parse_one(r#""\\\\\\\\\\\\\\\\\\""#),
        Atom::string(r#"\\\\\\\\\"#)
    );
}

#[test]
fn read_single_char_string() {
    fn single_char_string(s: &str) {
        assert_eq!(parse_one(&format!("\"{}\"", s)), Atom::string(s));
    }

    for c in "&-()*+,-/:;<=>?@[]^_`{}~!".chars() {
        single_char_string(&c.to_string());
    }
}

#[test]
fn read_erronous_input() {
    parse_has_error("(1 2");
    parse_has_error("[1 2");
    parse_has_error("\"abc");
    parse_has_error("\\");
    parse_has_error(r#"\\\\\\\\\\\\\\\\\\\"#);
    parse_has_error(r#"(1 \"abc"#);
    parse_has_error(r#"(1 \"abc\""#);
}

#[test]
fn read_quote() {
    assert_eq!(
        parse_one("'1"),
        create_list(&[Atom::symbol("quote"), Atom::integer(1)])
    );
    assert_eq!(
        parse_one("'(1 2 3)"),
        create_list(&[
            Atom::symbol("quote"),
            create_list(&[Atom::integer(1), Atom::integer(2), Atom::integer(3)])
        ])
    );
}

#[test]
fn read_quasiquote() {
    assert_eq!(
        parse_one("`1"),
        create_list(&[Atom::symbol("quasiquote"), Atom::integer(1)])
    );
    assert_eq!(
        parse_one("`(1 2 3)"),
        create_list(&[
            Atom::symbol("quasiquote"),
            create_list(&[Atom::integer(1), Atom::integer(2), Atom::integer(3)])
        ])
    );
}

#[test]
fn read_unquote() {
    assert_eq!(
        parse_one(",1"),
        create_list(&[Atom::symbol("unquote"), Atom::integer(1)])
    );
    assert_eq!(
        parse_one(",(1 2 3)"),
        create_list(&[
            Atom::symbol("unquote"),
            create_list(&[Atom::integer(1), Atom::integer(2), Atom::integer(3)])
        ])
    );
}

#[test]
fn read_unquote_quasiquote() {
    assert_eq!(
        parse_one("`(1 ,a 3)"),
        create_list(&[
            Atom::symbol("quasiquote"),
            create_list(&[
                Atom::integer(1),
                create_list(&[Atom::symbol("unquote"), Atom::symbol("a")]),
                Atom::integer(3)
            ])
        ])
    );
}

#[test]
fn read_unquote_splicing() {
    assert_eq!(
        parse_one(",@(1 2 3)"),
        create_list(&[
            Atom::symbol("unquote-splicing"),
            create_list(&[Atom::integer(1), Atom::integer(2), Atom::integer(3)])
        ])
    );
}

#[test]
fn arithmetic() {
    helper("(+ 1 2)", "3");
    helper("(+ 5 (* 2 3))", "11");
    helper("(- (+ 5 (* 2 3)) 3)", "8");
    helper("(/ (- (+ 5 (* 2 3)) 3) 4)", "2");
    helper("(/ (- (+ 515 (* 87 311)) 302) 27)", "1010");
    helper("(* -3 6)", "-18");
    helper("(/ (- (+ 515 (* -87 311)) 296) 27)", "-994");
}

#[test]
fn unbound_function() {
    run_has_error("(abc 1 2 3)");
}

#[test]
fn define() {
    helper("(define x 3)", "'x");
    helper("(define x 3) x", "3");

    helper("(define x 3)", "'x");
    helper("(define x 3) (define x 4)", "'x");
    helper("(define x 3) (define x 4) x", "4");

    helper("(define y (+ 1 7)) y", "8");

    run_has_error("(define w (abc))");
}

// //// //// //// // INTEGRATION TESTS // //// //// //// //

#[test]
fn can_load_standard_library() {
    let src = include_str!("../../lib/lib.lisp");
    run_code(src);
}
