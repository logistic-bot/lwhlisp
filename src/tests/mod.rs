use std::rc::Rc;

use chumsky::Parser;

use crate::{
    atom::Atom,
    env::Env,
    parsing::{lexer, parser},
};

#[test]
fn basic_functionnality() {
    let text = "(+ 1 1)";

    let mut env = Env::default();

    let tokens = lexer().parse(text).unwrap();
    let ast = parser().parse(tokens).unwrap();

    for atom in ast {
        let atom = Rc::new(atom);
        let result = Atom::eval(atom, &mut env).unwrap();

        assert_eq!(result.as_ref(), &Atom::integer(2))
    }
}

fn heavy_recursion() {
    let text = "
    (define (count n)\
        (if (= n 0)\
            0\
            (+ 1 (count (- n 1)))))\
    (count 10000)";

    let mut env = Env::default();

    let tokens = lexer().parse(text).unwrap();
    let ast = parser().parse(tokens).unwrap();

    let mut result = Rc::new(Atom::nil());
    for atom in ast {
        let atom = Rc::new(atom);
        result = Atom::eval(atom, &mut env).unwrap();
    }
    assert_eq!(result.as_ref(), &Atom::integer(2))
}
