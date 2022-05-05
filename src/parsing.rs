use ariadne::Span;
use chumsky::prelude::*;

#[derive(Debug, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    Symbol(String),
    PairSeparator,
}

fn symbol() -> impl Parser<char, String, Error = Simple<char>> {
    let id_start_char = one_of("abcdefghijklmnopqrstuvwxyz")
        .or(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))
        .or(one_of("+-*/%_=<>"))
        .labelled("symbol start character");
    let id_char = id_start_char
        .clone()
        .or(one_of("0123456789"))
        .or(one_of(":"))
        .labelled("symbol character");

    id_start_char
        .recover_with(skip_then_retry_until([]))
        .chain(id_char.repeated())
        .then_ignore(one_of(" ()").rewind())
        .collect::<String>()
        .labelled("symbol")
}

pub fn lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let open_paren = just('(').labelled("opening parenthesis");
    let close_paren = just(')').labelled("closing parenthesis");
    let pair_separator = just('.').labelled("pair separator");

    let symbol = symbol();

    let token = open_paren
        .map(|_| Token::OpenParen)
        .or(close_paren.map(|_| Token::CloseParen))
        .or(pair_separator.map(|_| Token::PairSeparator))
        .or(symbol.map(Token::Symbol))
        .labelled("Token");
    token.padded().repeated().then_ignore(end())
}
