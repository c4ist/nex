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

// ---------------------------------------------------------------------
// step 3.6: struct literals
// ---------------------------------------------------------------------

#[test]
fn parses_a_struct_literal() {
    assert_eq!(
        sexp("Point { x: 1.0, y: 2.0 }"),
        "(struct-lit Point (x 1) (y 2))"
    );
}

#[test]
fn parses_an_empty_struct_literal() {
    assert_eq!(sexp("Unit {}"), "(struct-lit Unit)");
}

#[test]
fn struct_literal_fields_allow_a_trailing_comma() {
    assert_eq!(sexp("P { x: 1, }"), "(struct-lit P (x 1))");
}

#[test]
fn struct_literal_field_values_are_full_expressions() {
    assert_eq!(sexp("P { x: a + b }"), "(struct-lit P (x (+ a b)))");
    assert_eq!(
        sexp("P { x: Q { y: 1 } }"),
        "(struct-lit P (x (struct-lit Q (y 1))))"
    );
}

#[test]
fn struct_literals_compose_with_postfix_and_infix() {
    assert_eq!(sexp("P { x: 1 }.x"), "(field (struct-lit P (x 1)) x)");
    assert_eq!(sexp("f(P { x: 1 })"), "(call f (struct-lit P (x 1)))");
}

#[test]
fn a_struct_literal_missing_its_colon_reports_an_error() {
    let (_, errors) = parse("P { x 1 }");
    assert_eq!(errors, vec!["expected `:`, found integer literal"]);
}

#[test]
fn an_unclosed_struct_literal_reports_an_error_without_hanging() {
    let (_, errors) = parse("P { x: 1");
    assert_eq!(errors, vec!["expected `}`, found end of file"]);
}

// ---------------------------------------------------------------------
// step 3.7: if/else as expressions
// ---------------------------------------------------------------------

#[test]
fn parses_an_if_without_an_else() {
    assert_eq!(sexp("if a { b }"), "(if a (block b))");
}

#[test]
fn parses_an_if_else() {
    assert_eq!(sexp("if a { b } else { c }"), "(if a (block b) (block c))");
}

// `else if` nests another if-expression in the else arm rather than
// introducing a dedicated chain node
#[test]
fn parses_an_else_if_chain() {
    assert_eq!(
        sexp("if a { b } else if c { d } else { e }"),
        "(if a (block b) (if c (block d) (block e)))"
    );
}

#[test]
fn if_conditions_are_full_expressions() {
    assert_eq!(sexp("if a + b > c { d }"), "(if (> (+ a b) c) (block d))");
    assert_eq!(sexp("if f(a) { b }"), "(if (call f a) (block b))");
}

// the reason the no_struct_literal flag exists: `if x { }` must be a
// condition plus a body, not a struct literal `x { }` with no body
#[test]
fn a_bare_ident_condition_is_not_a_struct_literal() {
    assert_eq!(sexp("if x { y }"), "(if x (block y))");
}

// ...but inside a delimiter the ambiguity is gone, so struct literals work
#[test]
fn struct_literals_still_parse_inside_a_condition_delimiter() {
    assert_eq!(
        sexp("if f(P { x: 1 }) { y }"),
        "(if (call f (struct-lit P (x 1))) (block y))"
    );
    assert_eq!(
        sexp("if (P { x: 1 }).x { y }"),
        "(if (field (struct-lit P (x 1)) x) (block y))"
    );
}

// and the restriction is scoped to the condition only - the body is a
// normal block again
#[test]
fn struct_literals_parse_again_inside_the_if_body() {
    assert_eq!(
        sexp("if a { P { x: 1 } }"),
        "(if a (block (struct-lit P (x 1))))"
    );
}

#[test]
fn if_is_an_expression_and_composes() {
    assert_eq!(sexp("{ if a { b } }"), "(block (if a (block b)))");
}

#[test]
fn an_if_missing_its_body_reports_an_error() {
    let (_, errors) = parse("if a");
    assert_eq!(errors, vec!["expected `{`, found end of file"]);
}

// ---------------------------------------------------------------------
// step 3.8: match expressions with all phase-2.6 patterns
// ---------------------------------------------------------------------

#[test]
fn parses_a_match_with_wildcard_and_binding_patterns() {
    assert_eq!(sexp("match x { _ => a }"), "(match x (_ a))");
    assert_eq!(sexp("match x { v => v }"), "(match x (v v))");
}

#[test]
fn parses_literal_patterns() {
    assert_eq!(
        sexp("match x { 1 => a, 2.5 => b, \"s\" => c, true => d }"),
        "(match x (1 a) (2.5 b) (\"s\" c) (true d))"
    );
}

// the minus belongs to the literal - a pattern has nothing to negate
#[test]
fn parses_negative_number_patterns() {
    assert_eq!(sexp("match x { -1 => a }"), "(match x (-1 a))");
    assert_eq!(sexp("match x { -2.5 => a }"), "(match x (-2.5 a))");
}

#[test]
fn parses_enum_variant_patterns() {
    assert_eq!(
        sexp("match x { Option::Some(v) => v, Option::None => z }"),
        "(match x ((Option::Some v) v) (Option::None z))"
    );
    // a single-segment name with a payload is still a variant
    assert_eq!(sexp("match x { Some(v) => v }"), "(match x ((Some v) v))");
}

#[test]
fn parses_struct_patterns() {
    assert_eq!(
        sexp("match p { Point { x: px, y: py } => px }"),
        "(match p ((struct-pat Point (x px) (y py)) px))"
    );
}

#[test]
fn parses_tuple_patterns() {
    assert_eq!(sexp("match t { (a, b) => a }"), "(match t ((tuple a b) a))");
}

#[test]
fn parses_nested_patterns() {
    assert_eq!(
        sexp("match x { Some((a, Point { x: 1 })) => a }"),
        "(match x ((Some (tuple a (struct-pat Point (x 1)))) a))"
    );
}

#[test]
fn match_arms_allow_a_trailing_comma() {
    assert_eq!(sexp("match x { _ => a, }"), "(match x (_ a))");
}

// `::` paths work in *patterns* but not yet in expressions, so the arms of
// the spec's sample match parse while its scrutinee does not. see
// docs/.../internals/ast-coverage.md - ExprKind has no path variant.
#[test]
fn match_arms_accept_the_spec_sample_patterns() {
    assert_eq!(
        sexp("match x { Option::Some(v) => print(v), Option::None => print(\"none\") }"),
        "(match x ((Option::Some v) (call print v)) (Option::None (call print \"none\")))"
    );
}

// known gap: a `::`-qualified *expression* has no ExprKind to parse into,
// so the spec's `match Option::Some(x) { .. }` scrutinee is rejected.
#[test]
fn a_path_qualified_scrutinee_is_not_supported_yet() {
    let (_, errors) = parse("match Option::Some(x) { _ => a }");
    assert_eq!(errors, vec!["expected `{`, found `::`"]);
}

#[test]
fn a_match_scrutinee_is_not_a_struct_literal() {
    assert_eq!(sexp("match x { _ => a }"), "(match x (_ a))");
}

#[test]
fn a_match_arm_missing_its_arrow_reports_an_error() {
    let (_, errors) = parse("match x { _ a }");
    assert_eq!(errors, vec!["expected `=>`, found identifier"]);
}

#[test]
fn an_unclosed_match_reports_an_error_without_hanging() {
    let (_, errors) = parse("match x { _ => a");
    assert_eq!(errors, vec!["expected `}`, found end of file"]);
}
