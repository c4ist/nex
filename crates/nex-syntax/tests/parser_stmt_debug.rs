//! statement parsing.

use nex_syntax::{parse_expr, print_expr};

/// statements are only reachable inside a block, so tests wrap their input
fn parse(src: &str) -> (String, Vec<String>) {
    let wrapped = format!("{{ {src} }}");
    let (tokens, lex_errors) = nex_lexer::tokenize(&wrapped);
    assert!(lex_errors.is_empty(), "unexpected lex errors in {src:?}");
    let (expr, errors) = parse_expr(&tokens);
    (
        print_expr(&expr),
        errors.into_iter().map(|e| e.message).collect(),
    )
}

#[track_caller]
fn sexp(src: &str) -> String {
    let (printed, errors) = parse(src);
    assert_eq!(errors, Vec::<String>::new(), "errors parsing {src:?}");
    printed
}

#[test]
fn parses_a_let_statement() {
    assert_eq!(sexp("let x = 5;"), "(block (let x 5))");
}

#[test]
fn parses_a_mutable_let_statement() {
    assert_eq!(sexp("let mut x = 5;"), "(block (let-mut x 5))");
}

#[test]
fn parses_a_let_with_a_type_annotation() {
    assert_eq!(sexp("let x: i32 = 5;"), "(block (let x i32 5))");
    assert_eq!(
        sexp("let mut s: str = \"hi\";"),
        "(block (let-mut s str \"hi\"))"
    );
}

#[test]
fn a_let_initializer_is_a_full_expression() {
    assert_eq!(sexp("let x = a + b * c;"), "(block (let x (+ a (* b c))))");
    assert_eq!(sexp("let x = f(1);"), "(block (let x (call f 1)))");
}

#[test]
fn parses_several_let_statements_in_a_block() {
    assert_eq!(
        sexp("let x = 1; let y = 2; x"),
        "(block (let x 1) (let y 2) x)"
    );
}

#[test]
fn parses_annotated_types() {
    assert_eq!(
        sexp("let a: Option<T> = x;"),
        "(block (let a (Option T) x))"
    );
    assert_eq!(sexp("let a: [i32] = x;"), "(block (let a (array i32) x))");
    assert_eq!(sexp("let a: &Point = x;"), "(block (let a (ref Point) x))");
    assert_eq!(
        sexp("let a: fn(i32, i32) -> i32 = x;"),
        "(block (let a (fn-type (i32 i32) i32) x))"
    );
    assert_eq!(sexp("let a: fn() = x;"), "(block (let a (fn-type ()) x))");
}

#[test]
fn parses_nested_type_annotations() {
    assert_eq!(
        sexp("let a: [&Option<T>] = x;"),
        "(block (let a (array (ref (Option T))) x))"
    );
}

#[test]
fn a_let_without_an_initializer_reports_an_error() {
    let (_, errors) = parse("let x;");
    assert_eq!(errors, vec!["expected `=`, found `;`"]);
}

#[test]
fn a_let_without_a_name_reports_an_error() {
    let (_, errors) = parse("let = 5;");
    assert_eq!(errors, vec!["expected identifier, found `=`"]);
}

#[test]
fn a_bad_type_annotation_reports_an_error() {
    let (_, errors) = parse("let x: 5 = 1;");
    assert_eq!(errors, vec!["expected a type, found integer literal"]);
}

// `>>` lexes as a single shift token, so the inner `>` of a nested generic
// closes nothing. needs the lexer or parser to split it.
#[test]
fn nested_generics_do_not_parse_yet() {
    let (_, errors) = parse("let a: Vec<Vec<T>> = x;");
    assert!(!errors.is_empty(), "expected `Vec<Vec<T>>` to fail for now");
}
