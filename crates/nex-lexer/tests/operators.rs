use nex_lexer::{tokenize_kinds, TokenKind};

/// The full operator/punctuation table. Each entry is `(source, expected)`.
const TABLE: &[(&str, TokenKind)] = &[
    ("+", TokenKind::Plus),
    ("-", TokenKind::Minus),
    ("*", TokenKind::Star),
    ("/", TokenKind::Slash),
    ("%", TokenKind::Percent),
    ("=", TokenKind::Eq),
    ("==", TokenKind::EqEq),
    ("!=", TokenKind::BangEq),
    ("<", TokenKind::Lt),
    ("<=", TokenKind::LtEq),
    (">", TokenKind::Gt),
    (">=", TokenKind::GtEq),
    ("&&", TokenKind::AmpAmp),
    ("||", TokenKind::PipePipe),
    ("!", TokenKind::Bang),
    ("&", TokenKind::Amp),
    ("|", TokenKind::Pipe),
    ("^", TokenKind::Caret),
    ("<<", TokenKind::Shl),
    (">>", TokenKind::Shr),
    ("+=", TokenKind::PlusEq),
    ("-=", TokenKind::MinusEq),
    ("*=", TokenKind::StarEq),
    ("/=", TokenKind::SlashEq),
    ("->", TokenKind::Arrow),
    ("=>", TokenKind::FatArrow),
    ("..", TokenKind::DotDot),
    ("..=", TokenKind::DotDotEq),
    ("::", TokenKind::ColonColon),
    (":", TokenKind::Colon),
    (";", TokenKind::Semi),
    (",", TokenKind::Comma),
    (".", TokenKind::Dot),
    ("(", TokenKind::LParen),
    (")", TokenKind::RParen),
    ("{", TokenKind::LBrace),
    ("}", TokenKind::RBrace),
    ("[", TokenKind::LBracket),
    ("]", TokenKind::RBracket),
];

#[test]
fn every_operator_lexes_as_a_single_token() {
    for (src, expected) in TABLE {
        assert_eq!(
            tokenize_kinds(src),
            vec![expected.clone(), TokenKind::Eof],
            "lexing {src:?}"
        );
    }
}

#[test]
fn longest_match_wins() {
    // If maximal munch were broken these would split into shorter operators.
    assert_eq!(
        tokenize_kinds("== != <= >= && || << >> += -= *= /= -> => .. ..= ::"),
        vec![
            TokenKind::EqEq,
            TokenKind::BangEq,
            TokenKind::LtEq,
            TokenKind::GtEq,
            TokenKind::AmpAmp,
            TokenKind::PipePipe,
            TokenKind::Shl,
            TokenKind::Shr,
            TokenKind::PlusEq,
            TokenKind::MinusEq,
            TokenKind::StarEq,
            TokenKind::SlashEq,
            TokenKind::Arrow,
            TokenKind::FatArrow,
            TokenKind::DotDot,
            TokenKind::DotDotEq,
            TokenKind::ColonColon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn operators_need_no_surrounding_whitespace() {
    assert_eq!(
        tokenize_kinds("a==b&&c!=d"),
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::EqEq,
            TokenKind::Ident("b".into()),
            TokenKind::AmpAmp,
            TokenKind::Ident("c".into()),
            TokenKind::BangEq,
            TokenKind::Ident("d".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn slash_slash_is_a_comment_not_two_slashes() {
    assert_eq!(
        tokenize_kinds("a // b"),
        vec![TokenKind::Ident("a".into()), TokenKind::Eof]
    );
}

#[test]
fn division_still_works() {
    assert_eq!(
        tokenize_kinds("a / b"),
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::Slash,
            TokenKind::Ident("b".into()),
            TokenKind::Eof
        ]
    );
}
