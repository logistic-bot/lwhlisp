use chumsky::prelude::*;

use crate::atom::Atom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    OpenParen,
    CloseParen,
    Symbol(String),
    Number(String),
    PairSeparator,
    Quote,
    QuasiQuote,
    Unquote,
    UnquoteSplicing,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::OpenParen => write!(f, "Opening parenthesis"),
            Token::CloseParen => write!(f, "Closing parenthesis"),
            Token::Symbol(s) => write!(f, "Symbol {}", s),
            Token::Number(n) => write!(f, "Number {}", n),
            Token::PairSeparator => write!(f, "Pair Separator"),
            Token::Quote => write!(f, "Quote"),
            Token::QuasiQuote => write!(f, "Quasiquote"),
            Token::Unquote => write!(f, "Unquote"),
            Token::UnquoteSplicing => write!(f, "Unquote-splicing"),
        }
    }
}

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
        .recover_with(skip_then_retry_until([]))
        .chain(id_char.repeated())
        .collect::<String>()
        .labelled("symbol")
}

pub fn lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let open_paren = just('(').labelled("opening parenthesis");
    let close_paren = just(')').labelled("closing parenthesis");
    let pair_separator = just('.').labelled("pair separator");
    let quote = just('\'').labelled("quote");
    let quasiquote = just('`').labelled("quasiquote");
    let unquote = just(',').labelled("unquote");
    let unquote_splicing = just(",@").labelled("unquote-splicing");

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
        .labelled("number");

    let symbol = symbol();

    let token = open_paren
        .map(|_| Token::OpenParen)
        .or(close_paren.map(|_| Token::CloseParen))
        .or(pair_separator.map(|_| Token::PairSeparator))
        .or(quote.map(|_| Token::Quote))
        .or(unquote_splicing.map(|_| Token::UnquoteSplicing))
        .or(unquote.map(|_| Token::Unquote))
        .or(quasiquote.map(|_| Token::QuasiQuote))
        .or(number.map(Token::Number))
        .or(symbol.map(Token::Symbol))
        .labelled("Token");
    token.padded().repeated().then_ignore(end())
}

pub fn parser() -> impl Parser<Token, Vec<Atom>, Error = Simple<Token>> {
    let atom = recursive(|atom| {
        let simple_atom = select! {
            Token::Number(x) => Atom::Number(x.parse().unwrap()),
            Token::Symbol(sym) => Atom::Symbol(sym),
        };

        let empty_list = just(Token::OpenParen)
            .then(just(Token::CloseParen))
            .ignored()
            .to(Atom::nil());

        let proper_list = just(Token::OpenParen)
            .ignore_then(atom.clone().repeated().at_least(1).map(|x| create_list(&x)))
            .then_ignore(just(Token::CloseParen));

        let improper_list = just(Token::OpenParen)
            .ignore_then(atom.clone().repeated().at_least(1))
            .then_ignore(just(Token::PairSeparator))
            .then(atom.clone())
            .then_ignore(just(Token::CloseParen))
            .map(|(atoms, last)| create_improper_list(&atoms, last));

        // let list = just(Token::OpenParen)
        //     .ignore_then(
        //         atom.clone().repeated().map(|x| create_list(&x)).or(atom
        //             .clone()
        //             .then_ignore(just(Token::PairSeparator))
        //             .then(atom.clone())
        //             .map(|(car, cdr)| Atom::cons(car, cdr))),
        //     )
        //     .then_ignore(just(Token::CloseParen));

        let list = empty_list.or(proper_list).or(improper_list);

        simple_atom
            .or(list)
            .or(just(Token::Quote).ignore_then(
                atom.clone()
                    .map(|a| Atom::cons(Atom::symbol("quote"), Atom::cons(a, Atom::nil()))),
            ))
            .or(just(Token::QuasiQuote).ignore_then(
                atom.clone()
                    .map(|a| Atom::cons(Atom::symbol("quasiquote"), Atom::cons(a, Atom::nil()))),
            ))
            .or(just(Token::Unquote).ignore_then(
                atom.clone()
                    .map(|a| Atom::cons(Atom::symbol("unquote"), Atom::cons(a, Atom::nil()))),
            ))
            .or(just(Token::UnquoteSplicing).ignore_then(
                atom.clone().map(|a| {
                    Atom::cons(Atom::symbol("unquote-splicing"), Atom::cons(a, Atom::nil()))
                }),
            ))
    });

    atom.repeated().then_ignore(end())
}

// converts a Vec<Atom> into a corresponding lisp cons list
fn create_list(x: &[Atom]) -> Atom {
    if let Some(first) = x.first().cloned() {
        Atom::cons(first, create_list(&x[1..]))
    } else {
        Atom::nil()
    }
}

fn create_improper_list(atoms: &[Atom], last: Atom) -> Atom {
    if let Some(first) = atoms.first().cloned() {
        Atom::cons(first, create_improper_list(&atoms[1..], last))
    } else {
        last
    }
}
