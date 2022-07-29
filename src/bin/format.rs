use chumsky::Parser as _;
use clap::Parser as _;
use color_eyre::{eyre::Context, Result};
use lwhlisp::{parsing::parser, print_parse_errs, read_file_to_string};

/// lwhlisp -- Lisp interpreter in Rust
/// Pretty-print a file
#[derive(clap::Parser, Debug)]
#[clap(author, version, about, propagate_version = true)]
struct Args {
    /// File to pretty-print
    #[clap(value_parser)]
    file: String,
    /// Replace the file with the formatted version
    #[clap(long)]
    replace: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let src = read_file_to_string(&args.file)?;
    let (atoms, errs) = parser().parse_recovery_verbose(src.trim());
    print_parse_errs(errs.clone(), src.trim());
    if errs.is_empty() {
        if let Some(atoms) = atoms {
            if args.replace {
                let out_file_path = format!("{}.tmp_format", args.file);
                let mut out_file = std::fs::File::create(&out_file_path)
                    .context("While creating temporary output file")?;
                for atom in atoms {
                    use std::io::Write;
                    writeln!(out_file, "{}\n", atom)
                        .context("While writing to temporary output file")?;
                }
                std::fs::rename(out_file_path, args.file)
                    .context("While moving formatted file to original")?;
            } else {
                for atom in atoms {
                    println!("{}\n", atom);
                }
            }
        }
    }

    Ok(())
}
