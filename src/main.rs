//! lwhlisp -- Lisp interpreter in Rust.

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// we use car and cdr a lot
#![allow(clippy::similar_names)]
// I find this clearer sometimes
#![allow(clippy::redundant_else)]
// I find this clearer sometimes
#![allow(clippy::use_self)]

use std::rc::Rc;

use chumsky::Parser as _;
use clap::Parser as _;
use color_eyre::{eyre::Context, Result};
use lwhlisp::{atom::Atom, env::Env, parsing::parser, print_parse_errs, read_file_to_string};
use tracing::{info, instrument};

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
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_writer(std::io::stderr)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let mut args = Args::parse();

    if args.files.is_empty() {
        info!("No files to execute, scheduling REPL start");
        args.repl = true;
    }

    let mut env = Env::default();

    if args.library.is_empty() {
        let default_library_path = String::from("lib/lib.lisp");
        info!("No library files given, adding default library {default_library_path}");
        args.library.push(default_library_path);
    }

    load_library(&args, &mut env)?;

    run_files(&args, &mut env)?;

    if args.repl {
        run_repl(env)?;
    }

    Ok(())
}

fn run_files(args: &Args, env: &mut Env) -> Result<(), color_eyre::Report> {
    for file in &args.files {
        run_file(file, env, args)?;
    }
    Ok(())
}

#[instrument(skip(args, env))]
fn run_file(file: &String, env: &mut Env, args: &Args) -> Result<(), color_eyre::Report> {
    info!("Running file '{file}'...");
    let src = read_file_to_string(file)?;

    let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
    print_parse_errs(errs, src.trim());

    if let Some(atoms) = atoms {
        for atom in atoms {
            let atom = Rc::new(atom);
            let result = Atom::eval(atom.clone(), env);
            match result {
                Ok(result) => {
                    if args.debug {
                        println!("{}", atom);
                        println!("=> {}", result);
                    }
                }
                Err(e) => {
                    eprintln!("{}\n!! {:?}", atom, e);
                }
            }
        }
    }

    info!("Done running file '{file}'!");

    Ok(())
}

fn load_library(args: &Args, env: &mut Env) -> Result<()> {
    for library_path in &args.library {
        load_library_file(library_path, env, args)?;
    }
    Ok(())
}

#[instrument(skip(args, env))]
fn load_library_file(
    library_path: &String,
    env: &mut Env,
    args: &Args,
) -> Result<(), color_eyre::Report> {
    info!("Loading library file '{library_path}'...");
    let src = read_file_to_string(library_path).context("While opening library file")?;

    let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
    print_parse_errs(errs, src.trim());

    if let Some(atoms) = atoms {
        for atom in atoms {
            let atom = Rc::new(atom);
            let result = Atom::eval(atom.clone(), env);
            match result {
                Ok(result) => {
                    if args.debug_library {
                        println!("{}", atom);
                        println!("=> {}", result);
                    }
                }
                Err(e) => {
                    eprintln!("{}\n!! {:?}", atom, e);
                }
            }
        }
    }

    info!("Done loading library file '{library_path}'!");

    Ok(())
}

/// Run a read-eval-print loop.
fn run_repl(mut env: Env) -> Result<()> {
    let mut rl = rustyline::Editor::<()>::new();
    let histfile = &".lisphistory.txt";
    drop(rl.load_history(histfile));
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
        let atom = Rc::new(atom);
        let result = Atom::eval(atom.clone(), env);
        match result {
            Ok(result) => {
                println!("=> {}", result);
            }
            Err(e) => {
                eprintln!("{}\n!! {:?}", atom, e);
            }
        }
    }
}
