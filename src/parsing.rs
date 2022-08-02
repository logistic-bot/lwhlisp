use chumsky::prelude::*;

use crate::atom::Atom;

fn symbol() -> impl Parser<char, String, Error = Simple<char>> {
    let id_start_char = one_of("abcdefghijklmnopqrstuvwxyz")
        .or(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))
        .or(one_of("+-*/%_=<>?"))
        .labelled("symbol start character");
    let id_char = id_start_char
        .clone()
        .or(one_of("0123456789"))
        .or(one_of(":"))
        .labelled("symbol character");

    id_start_char
        .chain(id_char.repeated())
        .collect::<String>()
        .labelled("symbol")
}

/// Parse a series of s-expressions.
///
/// # Panics
/// If the parser is incorrect about how to parse numbers, this may panic.
pub fn parser() -> impl Parser<char, Vec<Atom>, Error = Simple<char>> {
    let open_paren = just('(').labelled("opening parenthesis").padded();
    let close_paren = just(')').labelled("closing parenthesis").padded();
    let pair_separator = just('.').labelled("pair separator").padded();
    let quote = just('\'').labelled("quote").padded();
    let quasiquote = just('`').labelled("quasiquote").padded();
    let unquote = just(',').labelled("unquote").padded();
    let unquote_splicing = just(",@").labelled("unquote-splicing").padded();

    let frac = just('.').chain(text::digits(10));

    let exp = just('e')
        .or(just('E'))
        .chain(just('+').or(just('-')).or_not())
        .chain(text::digits(10));

    let number = just('-')
        .or_not()
        .chain(text::int(10))
        .chain(frac.or_not().flatten())
        .chain::<char, _, _>(exp.or_not().flatten())
        .collect::<String>()
        .labelled("number")
        .padded();

    let symbol = symbol().padded();

    let number = number.map(|x| Atom::Number(x.parse().unwrap()));
    let symbol = symbol.map(Atom::Symbol);

    let escape = just('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t'))
            .or(just('u').ignore_then(
                filter(char::is_ascii_hexdigit)
                    .repeated()
                    .exactly(4)
                    .collect::<String>()
                    .validate(|digits, span: _, emit| {
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    );

    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Atom::String)
        .labelled("string");

    let atom =
        recursive(|atom| {
            let empty_list = open_paren.then(close_paren).ignored().to(Atom::nil());

            let proper_list = open_paren
                .ignore_then(atom.clone().padded().repeated().at_least(1))
                .then_ignore(close_paren)
                .map(|x| create_list(&x));

            let improper_list = open_paren
                .ignore_then(atom.clone().padded().repeated().at_least(1))
                .then_ignore(pair_separator)
                .then(atom.clone().padded())
                .then_ignore(close_paren)
                .map(|(atoms, last)| create_improper_list(&atoms, last));

            let list = empty_list.or(proper_list).or(improper_list).padded();

            number
                .or(symbol)
                .or(string)
                .or(list)
                .or(quote.ignore_then(
                    atom.clone()
                        .padded()
                        .map(|a| Atom::cons(Atom::symbol("quote"), Atom::cons(a, Atom::nil()))),
                ))
                .or(quasiquote.ignore_then(
                    atom.clone().padded().map(|a| {
                        Atom::cons(Atom::symbol("quasiquote"), Atom::cons(a, Atom::nil()))
                    }),
                ))
                .or(unquote.ignore_then(
                    atom.clone()
                        .padded()
                        .map(|a| Atom::cons(Atom::symbol("unquote"), Atom::cons(a, Atom::nil()))),
                ))
                .or(unquote_splicing.ignore_then(atom.clone().padded().map(|a| {
                    Atom::cons(Atom::symbol("unquote-splicing"), Atom::cons(a, Atom::nil()))
                })))
        });

    atom.padded().repeated().then_ignore(end())
}

// converts a Vec<Atom> into a corresponding lisp cons list
fn create_list(x: &[Atom]) -> Atom {
    match x.first().cloned() {
        Some(first) => Atom::cons(first, create_list(&x[1..])),
        None => Atom::nil(),
    }
}

fn create_improper_list(atoms: &[Atom], last: Atom) -> Atom {
    if let Some(first) = atoms.first().cloned() {
        Atom::cons(first, create_improper_list(&atoms[1..], last))
    } else {
        last
    }
}
