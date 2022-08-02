//! lwhlisp is a lisp interpreter written in rust

#![warn(clippy::pedantic)]
// we use car and cdr a lot
#![allow(clippy::similar_names)]
// I find this clearer
#![allow(clippy::redundant_else)]

use std::{fs::File, io::Read};

use ariadne::{Color, Fmt, Label, Report, Source};
use chumsky::prelude::*;
use color_eyre::eyre::Context;
use tracing::info;

/// s-expressions and evaluating
pub mod atom;
/// Environment and data storage
pub mod env;
/// Parsing of s-expressions
pub mod parsing;

#[cfg(test)]
mod tests;

/// Convenience function to read a file to a string.
///
/// # Errors
/// If there is an error opening or reading the file, this will return an error.
pub fn read_file_to_string(path: &str) -> Result<String, color_eyre::Report> {
    let mut library_file = File::open(path).context(format!("While opening file {}", path))?;
    let mut src = String::new();
    library_file
        .read_to_string(&mut src)
        .context("While reading library file")?;
    Ok(src)
}

/// Pretty-print parse errors using ariadne.
///
/// # Panics
/// This may panic.
pub fn print_parse_errs(errs: Vec<Simple<char>>, src: &str) {
    for e in errs {
        let msg = if let chumsky::error::SimpleReason::Custom(msg) = e.reason() {
            msg.clone()
        } else {
            format!(
                "{}{}, expected {}",
                if e.found().is_some() {
                    "Unexpected token"
                } else {
                    "Unexpected end of input"
                },
                if let Some(label) = e.label() {
                    format!(" while parsing {}", label)
                } else {
                    String::new()
                },
                if e.expected().len() == 0 {
                    "something else".to_string()
                } else {
                    let res = e
                        .expected()
                        .map(|expected| match expected {
                            Some(expected) => expected.to_string(),
                            None => "end of input".to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    res
                },
            )
        };

        let label = Label::new(e.span()).with_message(match e.reason() {
            chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
            a => {
                info!("Unmatched error reason: {a:?}");
                format!(
                    "Unexpected {}",
                    e.found().map_or_else(
                        || "end of input".to_string(),
                        |c| format!("token {}", c.fg(Color::Red))
                    )
                )
            }
        });

        let report = Report::build(ariadne::ReportKind::Error, (), e.span().start)
            .with_code(0)
            .with_message(msg)
            .with_label(label);

        let report = match e.reason() {
            chumsky::error::SimpleReason::Unclosed { span, delimiter } => report.with_label(
                Label::new(span.clone())
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_color(Color::Yellow),
            ),
            // TODO: Maybe we could manage cusom resons better?
            chumsky::error::SimpleReason::Unexpected | chumsky::error::SimpleReason::Custom(_) => {
                report
            }
        };

        report.finish().eprint(Source::from(&src)).unwrap();
    }
}
