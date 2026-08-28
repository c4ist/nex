//! item parsing: whole files.

use nex_syntax::{parse_module, print_item};

fn parse(src: &str) -> (Vec<String>, Vec<String>) {
    let (tokens, lex_errors) = nex_lexer::tokenize(src);
    assert!(lex_errors.is_empty(), "unexpected lex errors in {src:?}");
    let (module, errors) = parse_module(&tokens);
    (
        module.items.iter().map(print_item).collect(),
        errors.into_iter().map(|e| e.message).collect(),
    )
}

#[track_caller]
fn items(src: &str) -> Vec<String> {
    let (items, errors) = parse(src);
    assert_eq!(errors, Vec::<String>::new(), "errors parsing {src:?}");
    items
}

#[test]
fn parses_an_empty_function() {
    assert_eq!(items("fn main() {}"), ["(fn main () () () (block))"]);
}

#[test]
fn parses_parameters_and_a_return_type() {
    assert_eq!(
        items("fn add(a: i32, b: i32) -> i32 { return a + b; }"),
        ["(fn add () ((a i32) (b i32)) i32 (block (return (+ a b))))"]
    );
}

#[test]
fn a_missing_return_type_is_unit() {
    assert_eq!(items("fn f(a: i32) {}"), ["(fn f () ((a i32)) () (block))"]);
}

#[test]
fn parses_generics() {
    assert_eq!(
        items("fn id<T>(x: T) -> T { x }"),
        ["(fn id (T) ((x T)) T (block x))"]
    );
    assert_eq!(
        items("fn pair<A, B>(a: A, b: B) {}"),
        ["(fn pair (A B) ((a A) (b B)) () (block))"]
    );
}

#[test]
fn parameter_lists_allow_a_trailing_comma() {
    assert_eq!(
        items("fn f(a: i32,) {}"),
        ["(fn f () ((a i32)) () (block))"]
    );
}

#[test]
fn parses_several_functions() {
    let out = items("fn a() {} fn b() {}");
    assert_eq!(out.len(), 2);
    assert_eq!(out[0], "(fn a () () () (block))");
    assert_eq!(out[1], "(fn b () () () (block))");
}

#[test]
fn an_empty_file_has_no_items() {
    assert_eq!(items(""), Vec::<String>::new());
}

#[test]
fn a_non_item_at_the_top_level_reports_an_error() {
    let (_, errors) = parse("let x = 1;");
    assert_eq!(errors[0], "expected an item, found `let`");
}

#[test]
fn a_function_missing_its_body_reports_an_error() {
    let (_, errors) = parse("fn f()");
    assert_eq!(errors, vec!["expected `{`, found end of file"]);
}

// the hello world in examples/, parsed as a whole file
#[test]
fn parses_the_hello_world_example() {
    let src = include_str!("../../../examples/hello.nex");
    let (items, errors) = parse(src);
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(
        items,
        ["(fn main () () () (block (call print \"hello, world\")))"]
    );
}
