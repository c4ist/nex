//! step 3.2: parsing leaf expressions (literals and identifiers).

use nex_syntax::{parse_expr, print_expr};

fn parse(src: &str) -> (String, Vec<String>) {
    let (tokens, lex_errors) = nex_lexer::tokenize(src);
    assert!(lex_errors.is_empty(), "unexpected lex errors in {src:?}");
    let (expr, errors) = parse_expr(&tokens);
    (
        print_expr(&expr),
        errors.into_iter().map(|e| e.message).collect(),
    )
}

#[test]
fn parses_an_integer_literal() {
    let (printed, errors) = parse("5");
    assert_eq!(printed, "5");
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parses_a_float_literal() {
    let (printed, errors) = parse("2.5");
    assert_eq!(printed, "2.5");
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parses_a_string_literal() {
    let (printed, errors) = parse("\"hi\"");
    assert_eq!(printed, "\"hi\"");
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn parses_bool_literals() {
    let (true_printed, true_errors) = parse("true");
    assert_eq!(true_printed, "true");
    assert_eq!(true_errors, Vec::<String>::new());

    let (false_printed, false_errors) = parse("false");
    assert_eq!(false_printed, "false");
    assert_eq!(false_errors, Vec::<String>::new());
}

#[test]
fn parses_an_identifier() {
    let (printed, errors) = parse("count");
    assert_eq!(printed, "count");
    assert_eq!(errors, Vec::<String>::new());
}

// not a valid expression start; recovers to a dummy unit node plus one error
#[test]
fn reports_an_error_on_a_non_expression_token() {
    let (printed, errors) = parse("+");
    assert_eq!(printed, "()");
    assert_eq!(errors, vec!["expected an expression, found `+`"]);
}

#[test]
fn reports_an_error_at_end_of_file() {
    let (printed, errors) = parse("");
    assert_eq!(printed, "()");
    assert_eq!(errors, vec!["expected an expression, found end of file"]);
}
