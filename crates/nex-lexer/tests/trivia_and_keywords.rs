use nex_lexer::{tokenize, tokenize_kinds, Span, TokenKind};

fn ident(name: &str) -> TokenKind {
    TokenKind::Ident(name.to_string())
}

#[test]
fn whitespace_and_line_comments_are_skipped() {
    assert_eq!(tokenize_kinds("  // hi\n"), vec![TokenKind::Eof]);
    assert_eq!(tokenize_kinds("\t\r\n   "), vec![TokenKind::Eof]);
    assert_eq!(tokenize_kinds("// only a comment"), vec![TokenKind::Eof]);
}

#[test]
fn comments_end_at_newline() {
    assert_eq!(
        tokenize_kinds("// comment\nlet"),
        vec![TokenKind::Let, TokenKind::Eof]
    );
}

#[test]
fn comment_between_tokens() {
    assert_eq!(
        tokenize_kinds("1 // two\n+ 3"),
        vec![
            TokenKind::Int(1),
            TokenKind::Plus,
            TokenKind::Int(3),
            TokenKind::Eof
        ]
    );
}

#[test]
fn fn_main_lexes_to_keyword_then_ident() {
    assert_eq!(
        tokenize_kinds("fn main"),
        vec![TokenKind::Fn, ident("main"), TokenKind::Eof]
    );
}

#[test]
fn every_keyword_is_recognised() {
    use TokenKind::*;
    let src = "fn let mut if else for while return struct enum match use mod pub true false in \
               break continue const type impl self";
    let expected = vec![
        Fn, Let, Mut, If, Else, For, While, Return, Struct, Enum, Match, Use, Mod, Pub, True,
        False, In, Break, Continue, Const, Type, Impl, SelfValue, Eof,
    ];
    assert_eq!(tokenize_kinds(src), expected);
}

#[test]
fn keyword_prefixes_are_identifiers() {
    assert_eq!(
        tokenize_kinds("iff letx returns Self _ _x x1"),
        vec![
            ident("iff"),
            ident("letx"),
            ident("returns"),
            ident("Self"),
            ident("_"),
            ident("_x"),
            ident("x1"),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn unicode_identifiers_are_allowed() {
    assert_eq!(tokenize_kinds("café"), vec![ident("café"), TokenKind::Eof]);
}

#[test]
fn spans_track_byte_offsets() {
    let (tokens, errors) = tokenize("let x = 1;");
    assert!(errors.is_empty());
    let spans: Vec<Span> = tokens.iter().map(|t| t.span).collect();
    assert_eq!(
        spans,
        vec![
            Span::new(0, 3),   // let
            Span::new(4, 5),   // x
            Span::new(6, 7),   // =
            Span::new(8, 9),   // 1
            Span::new(9, 10),  // ;
            Span::new(10, 10), // EOF
        ]
    );
}

#[test]
fn spans_slice_back_to_source() {
    let src = "fn add(a, b)";
    let (tokens, _) = tokenize(src);
    let texts: Vec<&str> = tokens
        .iter()
        .filter(|t| !t.is_eof())
        .map(|t| t.span.text(src))
        .collect();
    assert_eq!(texts, vec!["fn", "add", "(", "a", ",", "b", ")"]);
}

#[test]
fn spans_are_correct_after_multibyte_characters() {
    // "é" is two bytes, so the following token must start at offset 3.
    let src = "é x";
    let (tokens, errors) = tokenize(src);
    assert!(errors.is_empty());
    assert_eq!(tokens[0].span, Span::new(0, 2));
    assert_eq!(tokens[1].span, Span::new(3, 4));
}
