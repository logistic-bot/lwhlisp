use ariadne::{Color, Fmt, Label, Report, Source};
use chumsky::prelude::*;

pub mod atom;
pub mod env;
pub mod parsing;

#[cfg(test)]
mod tests;

/// Pretty-print parse errors using ariadne.
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
            _ => format!(
                "Unexpected {}",
                e.found()
                    .map(|c| format!("token {}", c.fg(Color::Red)))
                    .unwrap_or_else(|| "end of input".to_string())
            ),
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
            chumsky::error::SimpleReason::Unexpected => report,
            chumsky::error::SimpleReason::Custom(_) => report,
        };

        report.finish().eprint(Source::from(&src)).unwrap();
    }
}
