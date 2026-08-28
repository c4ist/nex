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

// assignment

#[test]
fn parses_a_plain_assignment() {
    assert_eq!(sexp("x = 5;"), "(block (= x 5))");
}

#[test]
fn parses_compound_assignments() {
    assert_eq!(sexp("x += 1;"), "(block (+= x 1))");
    assert_eq!(sexp("x -= 1;"), "(block (-= x 1))");
    assert_eq!(sexp("x *= 2;"), "(block (*= x 2))");
    assert_eq!(sexp("x /= 2;"), "(block (/= x 2))");
}

#[test]
fn assignment_targets_may_be_places() {
    assert_eq!(sexp("a.b = 1;"), "(block (= (field a b) 1))");
    assert_eq!(sexp("a[0] = 1;"), "(block (= (index a 0) 1))");
    assert_eq!(
        sexp("a.b[0].c = 1;"),
        "(block (= (field (index (field a b) 0) c) 1))"
    );
}

#[test]
fn an_assigned_value_is_a_full_expression() {
    assert_eq!(sexp("x = a + b * c;"), "(block (= x (+ a (* b c))))");
    assert_eq!(sexp("x += f(1);"), "(block (+= x (call f 1)))");
}

#[test]
fn assignments_mix_with_other_statements() {
    assert_eq!(sexp("let x = 1; x += 2; x"), "(block (let x 1) (+= x 2) x)");
}

// assignment is a statement, not an expression, so it can't nest
#[test]
fn assignment_does_not_chain() {
    let (_, errors) = parse("x = y = 1;");
    assert!(!errors.is_empty(), "expected `x = y = 1` to be rejected");
}

#[test]
fn an_assignment_without_a_value_reports_an_error() {
    let (_, errors) = parse("x = ;");
    assert_eq!(errors, vec!["expected an expression, found `;`"]);
}

// return, break, continue

#[test]
fn parses_return_with_a_value() {
    assert_eq!(sexp("return 5;"), "(block (return 5))");
    assert_eq!(sexp("return a + b;"), "(block (return (+ a b)))");
}

#[test]
fn parses_a_bare_return() {
    assert_eq!(sexp("return;"), "(block (return))");
}

// the value is optional, so `return` right before `}` is bare rather than
// swallowing the brace
#[test]
fn a_return_at_the_end_of_a_block_needs_no_semicolon() {
    assert_eq!(sexp("return"), "(block (return))");
}

#[test]
fn parses_break_and_continue() {
    assert_eq!(sexp("break;"), "(block (break))");
    assert_eq!(sexp("continue;"), "(block (continue))");
}

#[test]
fn control_flow_mixes_with_other_statements() {
    assert_eq!(
        sexp("let x = 1; x += 1; return x;"),
        "(block (let x 1) (+= x 1) (return x))"
    );
}

// loops

#[test]
fn parses_a_while_loop() {
    assert_eq!(sexp("while a { b; }"), "(block (while a (block b)))");
    assert_eq!(sexp("while true { }"), "(block (while true (block)))");
}

#[test]
fn parses_a_for_in_loop() {
    assert_eq!(
        sexp("for i in 0..10 { print(i); }"),
        "(block (for-in i (range 0 10) (block (call print i))))"
    );
}

#[test]
fn a_for_loop_iterates_any_expression() {
    assert_eq!(sexp("for x in xs { }"), "(block (for-in x xs (block)))");
    assert_eq!(
        sexp("for x in f(a) { }"),
        "(block (for-in x (call f a) (block)))"
    );
}

// same ambiguity as an if condition: `while x { }` is a loop, not a struct
// literal followed by nothing
#[test]
fn loop_headers_are_not_struct_literals() {
    assert_eq!(sexp("while x { }"), "(block (while x (block)))");
    assert_eq!(sexp("for i in x { }"), "(block (for-in i x (block)))");
}

#[test]
fn loops_nest_and_contain_statements() {
    assert_eq!(
        sexp("while a { for i in b { continue; } }"),
        "(block (while a (block (for-in i b (block (continue))))))"
    );
}

#[test]
fn loop_bodies_take_control_flow() {
    assert_eq!(
        sexp("while a { if b { break; } }"),
        "(block (while a (block (if b (block (break))))))"
    );
}

#[test]
fn a_for_loop_without_in_reports_an_error() {
    let (_, errors) = parse("for i 0..10 { }");
    assert_eq!(errors, vec!["expected `in`, found integer literal"]);
}

#[test]
fn a_while_without_a_body_reports_an_error() {
    // the helper wraps input in a block, so `}` is what turns up here
    let (_, errors) = parse("while a");
    assert_eq!(errors, vec!["expected `{`, found `}`"]);
}

// a statement ending in a braced body doesn't need a semicolon before the
// next one, the same rule rust uses
#[test]
fn block_like_statements_need_no_semicolon() {
    assert_eq!(sexp("while a { } b"), "(block (while a (block)) b)");
    assert_eq!(sexp("for i in x { } b"), "(block (for-in i x (block)) b)");
    assert_eq!(sexp("if a { } b"), "(block (if a (block)) b)");
    assert_eq!(sexp("match a { _ => b } c"), "(block (match a (_ b)) c)");
    assert_eq!(sexp("{ a } b"), "(block (block a) b)");
}

// ...but one is still allowed
#[test]
fn block_like_statements_still_accept_a_semicolon() {
    assert_eq!(sexp("while a { }; b"), "(block (while a (block)) b)");
    assert_eq!(sexp("if a { }; b"), "(block (if a (block)) b)");
}

// an ordinary expression without a semicolon is the block's value, so it
// still ends the block
#[test]
fn a_trailing_expression_still_ends_the_block() {
    assert_eq!(sexp("let x = 1; x"), "(block (let x 1) x)");
}

// the body of the sample program in the language spec
#[test]
fn parses_the_spec_sample_main_body() {
    let src = r#"
        let x = 5;
        let mut s = "hi";
        if x > 3 { print("big"); } else { print("small"); }
        for i in 0..10 { print(i); }
        let p = Point { x: 1.0, y: 2.0 };
        while x > 0 { x -= 1; }
        return x;
    "#;
    let (printed, errors) = parse(src);
    assert_eq!(errors, Vec::<String>::new());
    assert!(printed.contains("(for-in i (range 0 10)"), "{printed}");
    assert!(
        printed.contains("(struct-lit Point (x 1) (y 2))"),
        "{printed}"
    );
    assert!(
        printed.contains("(while (> x 0) (block (-= x 1)))"),
        "{printed}"
    );
    assert!(printed.ends_with("(return x))"), "{printed}");
}
