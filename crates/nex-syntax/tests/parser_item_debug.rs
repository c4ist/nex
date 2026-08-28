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

// structs

#[test]
fn parses_a_struct() {
    assert_eq!(
        items("struct Point { x: f64, y: f64 }"),
        ["(struct Point () ((x f64) (y f64)))"]
    );
}

#[test]
fn parses_an_empty_struct() {
    assert_eq!(items("struct Unit {}"), ["(struct Unit () ())"]);
}

#[test]
fn parses_a_generic_struct() {
    assert_eq!(
        items("struct Pair<A, B> { a: A, b: B }"),
        ["(struct Pair (A B) ((a A) (b B)))"]
    );
}

#[test]
fn struct_fields_allow_a_trailing_comma() {
    assert_eq!(items("struct S { a: i32, }"), ["(struct S () ((a i32)))"]);
}

#[test]
fn struct_fields_take_any_type() {
    assert_eq!(
        items("struct S { a: [i32], b: &T, c: Option<T> }"),
        ["(struct S () ((a (array i32)) (b (ref T)) (c (Option T))))"]
    );
}

#[test]
fn structs_and_functions_mix() {
    let out = items("struct P { x: i32 } fn f() {}");
    assert_eq!(out.len(), 2);
    assert_eq!(out[0], "(struct P () ((x i32)))");
}

#[test]
fn a_struct_field_missing_its_type_reports_an_error() {
    let (_, errors) = parse("struct S { a }");
    assert_eq!(errors, vec!["expected `:`, found `}`"]);
}

// a bad field type must not swallow the closing brace, or the struct loses
// its end and everything after it cascades
#[test]
fn a_bad_struct_field_type_keeps_the_closing_brace() {
    let (_, errors) = parse("struct S { a: 5 }");
    assert_eq!(errors, vec!["expected a type, found integer literal"]);
}
