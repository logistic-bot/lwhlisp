use std::{fs::File, io::Read};

use chumsky::Parser as _;
use clap::{Parser, Subcommand};
use color_eyre::{eyre::Context, Result};
use gc::Gc;
use lwhlisp::{atom::Atom, env::Env, parsing::parser, print_parse_errs};

/// lwhlisp -- Lisp interpreter in Rust
#[derive(clap::Parser, Debug)]
#[clap(author, version, about, propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a file or a REPL. If not FILE is give, run a REPL
    Run {
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
    },

    /// Pretty-print a file
    Format {
        /// File to pretty-print
        #[clap(value_parser)]
        file: String,
        /// Replace the file with the formatted version
        #[clap(long)]
        replace: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    match args.command {
        Commands::Run {
            library,
            files,
            repl,
            debug_library,
            debug,
        } => {
            subcommand_run(library, files, repl, debug_library, debug)?;
        }
        Commands::Format { file, replace } => {
            let src = read_file_to_string(&file)?;
            let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
            print_parse_errs(errs.clone(), src.trim());
            if errs.is_empty() {
                if let Some(atoms) = atoms {
                    if replace {
                        let out_file_path = format!("{}.tmp_format", file);
                        let mut out_file = std::fs::File::create(&out_file_path)
                            .context("While creating temporary output file")?;
                        for atom in atoms {
                            use std::io::Write;
                            writeln!(out_file, "{}\n", atom)
                                .context("While writing to temporary output file")?;
                        }
                        std::fs::rename(out_file_path, file)
                            .context("While moving formatted file to original")?;
                    } else {
                        for atom in atoms {
                            println!("{}\n", atom);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn subcommand_run(
    library: Vec<String>,
    files: Vec<String>,
    repl: bool,
    debug_library: bool,
    debug: bool,
) -> Result<(), color_eyre::Report> {
    let mut repl = repl;
    if files.is_empty() {
        repl = true;
    }

    let mut env = Env::default();

    let mut library = library;
    if library.is_empty() {
        library.push(String::from("lib/lib.lisp"));
    }

    for library_path in library {
        let src = read_file_to_string(&library_path)?;

        let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
        print_parse_errs(errs, src.trim());
        if let Some(atoms) = atoms {
            for atom in atoms {
                let atom = Gc::new(atom);
                let result = Atom::eval(atom.clone(), &mut env);
                match result {
                    Ok(result) => {
                        if debug_library {
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
    for file in files {
        let src = read_file_to_string(&file)?;
        let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
        print_parse_errs(errs, src.trim());
        if let Some(atoms) = atoms {
            for atom in atoms {
                let atom = Gc::new(atom);
                let result = Atom::eval(atom.clone(), &mut env);
                match result {
                    Ok(result) => {
                        if debug {
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

    if repl {
        run_repl(env)?;
    }
    Ok(())
}

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

fn eval_and_print_result(atoms: Vec<Atom>, env: &mut Env) {
    for atom in atoms {
        let atom = Gc::new(atom);
        let result = Atom::eval(atom.clone(), env);
        match result {
            Ok(result) => {
                println!("{}", atom);
                println!("=> {}", result);
            }
            Err(e) => {
                eprintln!("{}\n!! {:?}", atom, e)
            }
        }
    }
}

fn read_file_to_string(path: &str) -> Result<String, color_eyre::Report> {
    let mut library_file = File::open(path).context("While opening library file")?;
    let mut src = String::new();
    library_file
        .read_to_string(&mut src)
        .context("While reading library file")?;
    Ok(src)
}
