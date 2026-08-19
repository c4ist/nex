use nex_lexer::{tokenize, tokenize_kinds, LexErrorKind, TokenKind};

fn kinds_of_errors(src: &str) -> Vec<LexErrorKind> {
    tokenize(src).1.into_iter().map(|e| e.kind).collect()
}

// ------------------------------------------------------------------ integers

#[test]
fn decimal_integers() {
    assert_eq!(tokenize_kinds("0"), vec![TokenKind::Int(0), TokenKind::Eof]);
    assert_eq!(
        tokenize_kinds("42"),
        vec![TokenKind::Int(42), TokenKind::Eof]
    );
}

#[test]
fn underscore_separators_are_ignored() {
    assert_eq!(
        tokenize_kinds("1_000_000"),
        vec![TokenKind::Int(1_000_000), TokenKind::Eof]
    );
}

#[test]
fn hex_binary_and_octal() {
    assert_eq!(
        tokenize_kinds("0xFF"),
        vec![TokenKind::Int(255), TokenKind::Eof]
    );
    assert_eq!(
        tokenize_kinds("0x_ff"),
        vec![TokenKind::Int(255), TokenKind::Eof]
    );
    assert_eq!(
        tokenize_kinds("0b1010"),
        vec![TokenKind::Int(10), TokenKind::Eof]
    );
    assert_eq!(
        tokenize_kinds("0o17"),
        vec![TokenKind::Int(15), TokenKind::Eof]
    );
}

#[test]
fn radix_prefix_without_digits_is_an_error() {
    assert_eq!(kinds_of_errors("0x"), vec![LexErrorKind::MalformedNumber]);
    assert_eq!(kinds_of_errors("0b"), vec![LexErrorKind::MalformedNumber]);
}

#[test]
fn integer_overflow_is_reported() {
    assert_eq!(
        kinds_of_errors("99999999999999999999"),
        vec![LexErrorKind::IntegerOverflow]
    );
}

// -------------------------------------------------------------------- floats

#[test]
fn simple_floats() {
    assert_eq!(
        tokenize_kinds("1.0"),
        vec![TokenKind::Float(1.0), TokenKind::Eof]
    );
    assert_eq!(
        tokenize_kinds("0.5"),
        vec![TokenKind::Float(0.5), TokenKind::Eof]
    );
}

#[test]
fn exponents() {
    assert_eq!(
        tokenize_kinds("1e10"),
        vec![TokenKind::Float(1e10), TokenKind::Eof]
    );
    assert_eq!(
        tokenize_kinds("2.5e-3"),
        vec![TokenKind::Float(2.5e-3), TokenKind::Eof]
    );
    assert_eq!(
        tokenize_kinds("1E+5"),
        vec![TokenKind::Float(1e5), TokenKind::Eof]
    );
}

#[test]
fn range_after_integer_is_not_a_float() {
    assert_eq!(
        tokenize_kinds("0..10"),
        vec![
            TokenKind::Int(0),
            TokenKind::DotDot,
            TokenKind::Int(10),
            TokenKind::Eof
        ]
    );
    assert_eq!(
        tokenize_kinds("0..=10"),
        vec![
            TokenKind::Int(0),
            TokenKind::DotDotEq,
            TokenKind::Int(10),
            TokenKind::Eof
        ]
    );
}

#[test]
fn trailing_dot_before_identifier_is_malformed() {
    assert_eq!(
        kinds_of_errors("1.foo"),
        vec![LexErrorKind::MalformedNumber]
    );
}

#[test]
fn dangling_exponent_marker_is_not_consumed() {
    // `1e` is an integer followed by an identifier, not a broken float.
    assert_eq!(
        tokenize_kinds("1e"),
        vec![
            TokenKind::Int(1),
            TokenKind::Ident("e".into()),
            TokenKind::Eof
        ]
    );
    assert!(tokenize("1e").1.is_empty());
}

// ------------------------------------------------------------------- strings

#[test]
fn plain_string() {
    assert_eq!(
        tokenize_kinds("\"hello\""),
        vec![TokenKind::Str("hello".into()), TokenKind::Eof]
    );
}

#[test]
fn empty_string() {
    assert_eq!(
        tokenize_kinds("\"\""),
        vec![TokenKind::Str(String::new()), TokenKind::Eof]
    );
}

#[test]
fn escape_sequences() {
    let src = r#""a\nb\tc\\d\"e\0f""#;
    assert_eq!(
        tokenize_kinds(src),
        vec![TokenKind::Str("a\nb\tc\\d\"e\0f".into()), TokenKind::Eof]
    );
}

#[test]
fn hex_escapes() {
    assert_eq!(
        tokenize_kinds(r#""\x41\x62""#),
        vec![TokenKind::Str("Ab".into()), TokenKind::Eof]
    );
}

#[test]
fn bad_hex_escape_is_reported() {
    assert_eq!(
        kinds_of_errors(r#""\xZZ""#),
        vec![LexErrorKind::InvalidHexEscape]
    );
    assert_eq!(
        kinds_of_errors(r#""\x4""#),
        vec![LexErrorKind::InvalidHexEscape]
    );
}

#[test]
fn unknown_escape_is_reported_but_recovers() {
    let (tokens, errors) = tokenize(r#""a\qb""#);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, LexErrorKind::InvalidEscape('q'));
    assert_eq!(tokens[0].kind, TokenKind::Str("aqb".into()));
}

#[test]
fn unterminated_string_at_eof() {
    let (tokens, errors) = tokenize("\"oops");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, LexErrorKind::UnterminatedString);
    assert_eq!(tokens[0].kind, TokenKind::Str("oops".into()));
}

#[test]
fn unterminated_string_stops_at_newline_and_recovers() {
    let (tokens, errors) = tokenize("\"oops\nlet");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, LexErrorKind::UnterminatedString);
    // Scanning continues on the next line.
    assert_eq!(tokens[1].kind, TokenKind::Let);
}
