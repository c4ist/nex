//! parsing expressions: leaves (step 3.2) and the pratt operator ladder
//! (step 3.3).

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

/// parses `src` and asserts it produced no errors, returning the s-expr
#[track_caller]
fn sexp(src: &str) -> String {
    let (printed, errors) = parse(src);
    assert_eq!(errors, Vec::<String>::new(), "errors parsing {src:?}");
    printed
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

// ---------------------------------------------------------------------
// step 3.3: operator precedence and associativity
// ---------------------------------------------------------------------

#[test]
fn multiplication_binds_tighter_than_addition() {
    assert_eq!(sexp("a + b * c"), "(+ a (* b c))");
    assert_eq!(sexp("a * b + c"), "(+ (* a b) c)");
}

#[test]
fn and_binds_tighter_than_or() {
    assert_eq!(sexp("a && b || c"), "(|| (&& a b) c)");
    assert_eq!(sexp("a || b && c"), "(|| a (&& b c))");
}

#[test]
fn arithmetic_operators_are_left_associative() {
    assert_eq!(sexp("a - b - c"), "(- (- a b) c)");
    assert_eq!(sexp("a / b / c"), "(/ (/ a b) c)");
}

#[test]
fn prefix_operators_bind_tighter_than_any_infix() {
    assert_eq!(sexp("-a * b"), "(* (- a) b)");
    assert_eq!(sexp("!a && b"), "(&& (! a) b)");
    // ...but the operand of a prefix op is itself only a prefix expression,
    // so the infix operator still wins the wider expression
    assert_eq!(sexp("-a + -b"), "(+ (- a) (- b))");
}

#[test]
fn prefix_operators_stack() {
    assert_eq!(sexp("--a"), "(- (- a))");
    assert_eq!(sexp("!!a"), "(! (! a))");
    assert_eq!(sexp("-!a"), "(- (! a))");
}

#[test]
fn comparison_binds_looser_than_arithmetic() {
    assert_eq!(sexp("a + b < c * d"), "(< (+ a b) (* c d))");
    assert_eq!(sexp("a == b + c"), "(== a (+ b c))");
}

#[test]
fn bitwise_ladder_orders_or_xor_and_shift() {
    // | looser than ^ looser than & looser than << looser than +
    assert_eq!(sexp("a | b ^ c"), "(| a (^ b c))");
    assert_eq!(sexp("a ^ b & c"), "(^ a (& b c))");
    assert_eq!(sexp("a & b << c"), "(& a (<< b c))");
    assert_eq!(sexp("a << b + c"), "(<< a (+ b c))");
}

// one expression exercising every level of the ladder at once, loosest to
// tightest: || && == | ^ & << + *
#[test]
fn the_full_precedence_ladder_nests_correctly() {
    assert_eq!(
        sexp("a || b && c == d | e ^ f & g << h + i * j"),
        "(|| a (&& b (== c (| d (^ e (& f (<< g (+ h (* i j)))))))))"
    );
}

// comparisons are left-associative here rather than non-associative as in
// rust; `a < b == c` parses instead of erroring. revisit if the type
// checker makes chained comparisons confusing rather than merely useless.
#[test]
fn comparisons_chain_left_associatively() {
    assert_eq!(sexp("a < b == c"), "(== (< a b) c)");
}

#[test]
fn a_missing_right_operand_reports_one_error() {
    let (printed, errors) = parse("a +");
    assert_eq!(printed, "(+ a ())");
    assert_eq!(errors, vec!["expected an expression, found end of file"]);
}

// ---------------------------------------------------------------------
// step 3.4: parentheses and blocks-as-expressions
// ---------------------------------------------------------------------

#[test]
fn parentheses_override_precedence() {
    assert_eq!(sexp("(a + b) * c"), "(* (+ a b) c)");
    assert_eq!(sexp("a * (b + c)"), "(* a (+ b c))");
    assert_eq!(sexp("-(a + b)"), "(- (+ a b))");
}

#[test]
fn redundant_parentheses_leave_no_trace() {
    // parens group, they don't produce a node of their own
    assert_eq!(sexp("((a))"), "a");
    assert_eq!(sexp("(a + b)"), "(+ a b)");
}

#[test]
fn empty_parentheses_are_the_unit_literal() {
    assert_eq!(sexp("()"), "()");
}

#[test]
fn parses_an_empty_block() {
    assert_eq!(sexp("{}"), "(block)");
}

#[test]
fn parses_a_block_of_expression_statements() {
    assert_eq!(sexp("{ a }"), "(block a)");
    assert_eq!(sexp("{ a; b }"), "(block a b)");
    // a trailing semicolon is allowed
    assert_eq!(sexp("{ a; b; }"), "(block a b)");
}

#[test]
fn blocks_nest_and_compose_with_operators() {
    assert_eq!(sexp("{ { a } }"), "(block (block a))");
    assert_eq!(sexp("{ a + b }"), "(block (+ a b))");
}

#[test]
fn an_unclosed_paren_reports_an_error_without_hanging() {
    let (_, errors) = parse("(a + b");
    assert_eq!(errors, vec!["expected `)`, found end of file"]);
}

// regression: `advance` is a no-op at Eof, so an unterminated block must
// break out explicitly or the statement loop spins forever
#[test]
fn an_unclosed_block_reports_an_error_without_hanging() {
    let (printed, errors) = parse("{ a; b");
    assert_eq!(printed, "(block a b)");
    assert_eq!(errors, vec!["expected `}`, found end of file"]);
}

#[test]
fn an_unclosed_empty_block_reports_an_error_without_hanging() {
    let (printed, errors) = parse("{");
    assert_eq!(printed, "(block)");
    assert_eq!(errors, vec!["expected `}`, found end of file"]);
}

// ---------------------------------------------------------------------
// step 3.5: call, field-access and index postfix chains
// ---------------------------------------------------------------------

#[test]
fn parses_calls_with_various_arities() {
    assert_eq!(sexp("f()"), "(call f)");
    assert_eq!(sexp("f(a)"), "(call f a)");
    assert_eq!(sexp("f(a, b)"), "(call f a b)");
    // a trailing comma is allowed
    assert_eq!(sexp("f(a, b,)"), "(call f a b)");
}

#[test]
fn parses_field_access_and_indexing() {
    assert_eq!(sexp("a.b"), "(field a b)");
    assert_eq!(sexp("a.b.c"), "(field (field a b) c)");
    assert_eq!(sexp("a[0]"), "(index a 0)");
    assert_eq!(sexp("a[0][1]"), "(index (index a 0) 1)");
}

// the plan's worked example for this step
#[test]
fn parses_a_mixed_postfix_chain() {
    assert_eq!(
        sexp("f(x)(y).z[0]"),
        "(index (field (call (call f x) y) z) 0)"
    );
}

// postfix binds tighter than prefix, so the negation applies to the call's
// result rather than to `a`
#[test]
fn postfix_binds_tighter_than_prefix() {
    assert_eq!(sexp("-a.b()"), "(- (call (field a b)))");
    assert_eq!(sexp("!f(a)"), "(! (call f a))");
}

#[test]
fn postfix_binds_tighter_than_any_infix() {
    assert_eq!(sexp("a + f(b)"), "(+ a (call f b))");
    assert_eq!(sexp("a.b * c.d"), "(* (field a b) (field c d))");
}

#[test]
fn call_arguments_are_full_expressions() {
    assert_eq!(sexp("f(a + b, c.d)"), "(call f (+ a b) (field c d))");
    assert_eq!(sexp("f(g(a))"), "(call f (call g a))");
}

#[test]
fn a_non_identifier_after_dot_reports_an_error() {
    let (_, errors) = parse("a.0");
    assert_eq!(errors, vec!["expected identifier, found integer literal"]);
}

#[test]
fn unclosed_postfix_brackets_report_errors_without_hanging() {
    let (_, errors) = parse("f(a");
    assert_eq!(errors, vec!["expected `)`, found end of file"]);

    let (_, errors) = parse("a[0");
    assert_eq!(errors, vec!["expected `]`, found end of file"]);

    // an unterminated argument list must not spin: at Eof neither
    // parse_expr nor expect consumes anything
    let (_, errors) = parse("f(a,");
    assert_eq!(errors, vec!["expected `)`, found end of file"]);
}
