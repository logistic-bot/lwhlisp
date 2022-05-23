use std::rc::Rc;

use chumsky::Parser;
use color_eyre::Result;
use lwhlisp::{
    atom::Atom,
    env::Env,
    parsing::{lexer, parser},
    print_lex_errs, print_parse_errs,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut rl = rustyline::Editor::<()>::new();
    let mut env = Env::default();

    println!("Loading standard library...");
    let src = include_str!("../lib/lib.lisp");
    let (tokens, errs) = lexer().parse_recovery(src.trim());
    print_lex_errs(errs, src);
    if let Some(tokens) = tokens {
        let (atoms, errs) = parser().parse_recovery(tokens);
        if let Some(atoms) = atoms {
            for atom in atoms {
                let atom = Rc::new(atom);
                let result = Atom::eval(atom.clone(), &mut env);
                match result {
                    Ok(result) => {
                        println!("{} => {}", atom, result);
                    }
                    Err(e) => {
                        eprintln!("{} !! {:?}", atom, e)
                    }
                }
            }
        }
        print_parse_errs(errs, src);
    }
    println!("Finished.");

    let histfile = &".lisphistory.txt";
    let _ = rl.load_history(histfile);
    loop {
        let readline = rl.readline("user> ");
        match readline {
            Err(_) => break,
            Ok(src) => {
                rl.add_history_entry(&src);
                let (tokens, errs) = lexer().parse_recovery(src.trim());
                print_lex_errs(errs, &src);
                if let Some(tokens) = tokens {
                    let (atoms, errs) = parser().parse_recovery(tokens);
                    if let Some(atoms) = atoms {
                        for atom in atoms {
                            let atom = Rc::new(atom);
                            let result = Atom::eval(atom.clone(), &mut env);
                            match result {
                                Ok(result) => {
                                    println!("{} => {}", atom, result);
                                }
                                Err(e) => {
                                    eprintln!("{} !! {:?}", atom, e)
                                }
                            }
                        }
                    }
                    print_parse_errs(errs, &src);
                }
            }
        }
    }

    rl.save_history(histfile)?;

    Ok(())
}
