use std::{fs::File, io::Read};

use chumsky::Parser;
use color_eyre::{eyre::Context, Result};
use gc::Gc;
use lwhlisp::{atom::Atom, env::Env, parsing::parser, print_parse_errs};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut rl = rustyline::Editor::<()>::new();
    let mut env = Env::default();

    println!("Loading standard library...");
    let src = {
        let mut library_file = File::open("lib/lib.lisp").context("While opening library file")?;
        let mut src = String::new();
        library_file
            .read_to_string(&mut src)
            .context("While reading library file")?;
        src
    };

    let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
    print_parse_errs(errs, src.trim());
    if let Some(atoms) = atoms {
        for atom in atoms {
            let atom = Gc::new(atom);
            let result = Atom::eval(atom.clone(), &mut env);
            match result {
                Ok(result) => {
                    println!("{}\n=> {}\n", atom, result);
                }
                Err(e) => {
                    eprintln!("{}\n!! {:?}", atom, e)
                }
            }
        }
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
                let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
                print_parse_errs(errs, src.trim());
                if let Some(atoms) = atoms {
                    for atom in atoms {
                        let atom = Gc::new(atom);
                        let result = Atom::eval(atom.clone(), &mut env);
                        match result {
                            Ok(result) => {
                                println!("{}\n=> {}", atom, result);
                            }
                            Err(e) => {
                                eprintln!("{}\n!! {:?}", atom, e)
                            }
                        }
                    }
                }
            }
        }
    }

    rl.save_history(histfile)?;

    Ok(())
}
