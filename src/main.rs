//! lwhlisp -- Lisp interpreter in Rust.

use chumsky::Parser as _;
use clap::Parser as _;
use color_eyre::{eyre::Context, Result};
use gc::Gc;
use lwhlisp::{atom::Atom, env::Env, parsing::parser, print_parse_errs, read_file_to_string};

/// lwhlisp -- Lisp interpreter in Rust
/// Run a file or a REPL. If not FILE is give, run a REPL
#[derive(clap::Parser, Debug)]
#[clap(author, version, about, propagate_version = true)]
struct Args {
    /// Overide library files to evaluate at startup
    #[clap(long)]
    library: Vec<String>,

    /// Files to evaluate
    #[clap(short, long)]
    files: Vec<String>,

    /// Start a REPL. Implied if no FILE is given
    #[clap(long)]
    repl: bool,

    /// Show debugging information in the library
    #[clap(long)]
    debug_library: bool,

    /// Show debugging information in evaluated files
    #[clap(long)]
    debug: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = Args::parse();

    if args.files.is_empty() {
        args.repl = true;
    }

    let mut env = Env::default();

    if args.library.is_empty() {
        args.library.push(String::from("lib/lib.lisp"));
    }

    for library_path in args.library {
        let src = read_file_to_string(&library_path).context("While opening library file")?;

        let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
        print_parse_errs(errs, src.trim());
        if let Some(atoms) = atoms {
            for atom in atoms {
                let atom = Gc::new(atom);
                let result = Atom::eval(atom.clone(), &mut env);
                match result {
                    Ok(result) => {
                        if args.debug_library {
                            println!("{}", atom);
                            println!("=> {}", result);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}\n!! {:?}", atom, e)
                    }
                }
            }
        }
    }
    for file in args.files {
        let src = read_file_to_string(&file)?;
        let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
        print_parse_errs(errs, src.trim());
        if let Some(atoms) = atoms {
            for atom in atoms {
                let atom = Gc::new(atom);
                let result = Atom::eval(atom.clone(), &mut env);
                match result {
                    Ok(result) => {
                        if args.debug {
                            println!("{}", atom);
                            println!("=> {}", result);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}\n!! {:?}", atom, e)
                    }
                }
            }
        }
    }

    if args.repl {
        run_repl(env)?;
    }

    Ok(())
}

/// Run a read-eval-print loop.
fn run_repl(mut env: Env) -> Result<(), color_eyre::Report> {
    let mut rl = rustyline::Editor::<()>::new();
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
                    eval_and_print_result(atoms, &mut env);
                }
            }
        }
    }
    rl.save_history(histfile)?;
    Ok(())
}

/// Eval atoms and print the result.
///
/// Will evaluate the given atoms in order, and print stack traces on error.
fn eval_and_print_result(atoms: Vec<Atom>, env: &mut Env) {
    for atom in atoms {
        let atom = Gc::new(atom);
        let result = Atom::eval(atom.clone(), env);
        match result {
            Ok(result) => {
                println!("=> {}", result);
            }
            Err(e) => {
                eprintln!("{}\n!! {:?}", atom, e)
            }
        }
    }
}
