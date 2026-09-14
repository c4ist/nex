//! expression parsing.

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

// operator precedence and associativity

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
    assert_eq!(sexp("a | b ^ c"), "(| a (^ b c))");
    assert_eq!(sexp("a ^ b & c"), "(^ a (& b c))");
    assert_eq!(sexp("a & b << c"), "(& a (<< b c))");
    assert_eq!(sexp("a << b + c"), "(<< a (+ b c))");
}

// every level of the ladder at once
#[test]
fn the_full_precedence_ladder_nests_correctly() {
    assert_eq!(
        sexp("a || b && c == d | e ^ f & g << h + i * j"),
        "(|| a (&& b (== c (| d (^ e (& f (<< g (+ h (* i j)))))))))"
    );
}

// left-associative, unlike rust where this is an error
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

// parentheses and blocks-as-expressions

#[test]
fn parentheses_override_precedence() {
    assert_eq!(sexp("(a + b) * c"), "(* (+ a b) c)");
    assert_eq!(sexp("a * (b + c)"), "(* a (+ b c))");
    assert_eq!(sexp("-(a + b)"), "(- (+ a b))");
}

#[test]
fn redundant_parentheses_leave_no_trace() {
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

// advance is a no-op at Eof, so the statement loop needs its own Eof case
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

// call, field-access and index postfix chains

#[test]
fn parses_calls_with_various_arities() {
    assert_eq!(sexp("f()"), "(call f)");
    assert_eq!(sexp("f(a)"), "(call f a)");
    assert_eq!(sexp("f(a, b)"), "(call f a b)");
    assert_eq!(sexp("f(a, b,)"), "(call f a b)");
}

#[test]
fn parses_field_access_and_indexing() {
    assert_eq!(sexp("a.b"), "(field a b)");
    assert_eq!(sexp("a.b.c"), "(field (field a b) c)");
    assert_eq!(sexp("a[0]"), "(index a 0)");
    assert_eq!(sexp("a[0][1]"), "(index (index a 0) 1)");
}

#[test]
fn parses_a_mixed_postfix_chain() {
    assert_eq!(
        sexp("f(x)(y).z[0]"),
        "(index (field (call (call f x) y) z) 0)"
    );
}

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

    let (_, errors) = parse("f(a,");
    assert_eq!(errors, vec!["expected `)`, found end of file"]);
}

// struct literals

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

// if/else as expressions

#[test]
fn parses_an_if_without_an_else() {
    assert_eq!(sexp("if a { b }"), "(if a (block b))");
}

#[test]
fn parses_an_if_else() {
    assert_eq!(sexp("if a { b } else { c }"), "(if a (block b) (block c))");
}

// `else if` just nests another if in the else arm
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

// `if x { }` is a condition plus a body, not the struct literal `x { }`
#[test]
fn a_bare_ident_condition_is_not_a_struct_literal() {
    assert_eq!(sexp("if x { y }"), "(if x (block y))");
}

// inside a delimiter the ambiguity is gone
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

// the restriction covers the condition only
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

// match expressions with all phase-2.6 patterns

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

// the minus belongs to the literal
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

#[test]
fn match_arms_accept_the_spec_sample_patterns() {
    assert_eq!(
        sexp("match x { Option::Some(v) => print(v), Option::None => print(\"none\") }"),
        "(match x ((Option::Some v) (call print v)) (Option::None (call print \"none\")))"
    );
}

// known gap: ExprKind has no path variant, so `Option::Some(x)` only works
// as a pattern, not as an expression
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

// range expressions

#[test]
fn parses_exclusive_and_inclusive_ranges() {
    assert_eq!(sexp("0..10"), "(range 0 10)");
    assert_eq!(sexp("0..=10"), "(range-incl 0 10)");
    assert_eq!(sexp("a..b"), "(range a b)");
}

#[test]
fn a_range_between_integers_is_not_a_float() {
    assert_eq!(sexp("0..10"), "(range 0 10)");
}

#[test]
fn ranges_bind_looser_than_arithmetic_and_logic() {
    assert_eq!(sexp("a + 1..b * 2"), "(range (+ a 1) (* b 2))");
    assert_eq!(sexp("a..b || c"), "(range a (|| b c))");
}

#[test]
fn range_endpoints_may_be_postfix_chains() {
    assert_eq!(sexp("a.lo..f(b)"), "(range (field a lo) (call f b))");
}

#[test]
fn ranges_appear_inside_calls_and_indexes() {
    assert_eq!(sexp("f(0..n)"), "(call f (range 0 n))");
    assert_eq!(sexp("a[0..n]"), "(index a (range 0 n))");
}

// `a..b..c` yields `a..b` and leaves `..c` unconsumed
#[test]
fn ranges_do_not_chain() {
    assert_eq!(sexp("a..b..c"), "(range a b)");
}

// known gap: Range needs both endpoints, so `a..` and `..b` can't be built
#[test]
fn open_ended_ranges_are_not_supported_yet() {
    let (_, errors) = parse("a..");
    assert_eq!(errors, vec!["expected an expression, found end of file"]);
}

// error recovery

#[test]
fn three_broken_statements_report_three_errors() {
    let (_, errors) = parse("{ else; else; else }");
    assert_eq!(
        errors,
        vec![
            "expected an expression, found `else`",
            "expected an expression, found `else`",
            "expected an expression, found `else`",
        ]
    );
}

#[test]
fn recovery_skips_the_rest_of_a_broken_statement() {
    let (_, errors) = parse("{ else a b c; d }");
    assert_eq!(errors, vec!["expected an expression, found `else`"]);
}

#[test]
fn parsing_continues_after_a_recovered_error() {
    let (printed, errors) = parse("{ else; a + b }");
    assert_eq!(errors, vec!["expected an expression, found `else`"]);
    assert!(printed.contains("(+ a b)"), "{printed}");
}

#[test]
fn recovery_stops_at_the_closing_brace() {
    let (_, errors) = parse("{ else }");
    assert_eq!(errors, vec!["expected an expression, found `else`"]);
}

#[test]
fn recovery_terminates_at_end_of_file() {
    let (_, errors) = parse("{ else");
    assert_eq!(
        errors,
        vec![
            "expected an expression, found `else`",
            "expected `}`, found end of file",
        ]
    );
}

// closures

#[test]
fn parses_a_closure() {
    assert_eq!(sexp("|x| x + 1"), "(closure ((x _)) (+ x 1))");
}

// `||` is one token, so an empty parameter list is its own case
#[test]
fn parses_a_closure_with_no_parameters() {
    assert_eq!(sexp("|| 0"), "(closure () 0)");
}

#[test]
fn parses_closure_parameters() {
    assert_eq!(sexp("|a, b| a"), "(closure ((a _) (b _)) a)");
    assert_eq!(sexp("|a, b,| a"), "(closure ((a _) (b _)) a)");
}

// a parameter can be annotated, and an unannotated one prints as `_`
#[test]
fn parses_annotated_closure_parameters() {
    assert_eq!(sexp("|a: i32, b: str| a"), "(closure ((a i32) (b str)) a)");
    assert_eq!(sexp("|a: i32, b| a"), "(closure ((a i32) (b _)) a)");
}

#[test]
fn parses_a_closure_with_a_block_body() {
    assert_eq!(sexp("|n| { n * 2 }"), "(closure ((n _)) (block (* n 2)))");
}

// the body takes as much as it can, so this is one closure returning a sum
#[test]
fn a_closure_body_extends_as_far_as_it_can() {
    assert_eq!(sexp("|x| x + 1 + 2"), "(closure ((x _)) (+ (+ x 1) 2))");
}

#[test]
fn closures_nest() {
    assert_eq!(
        sexp("|a| |b| a + b"),
        "(closure ((a _)) (closure ((b _)) (+ a b)))"
    );
}

#[test]
fn a_closure_can_be_an_argument() {
    assert_eq!(sexp("f(|x| x, 1)"), "(call f (closure ((x _)) x) 1)");
}

// `|` is still bitwise or when it isn't starting an expression
#[test]
fn pipe_is_still_an_operator_between_expressions() {
    assert_eq!(sexp("1 | 2"), "(| 1 2)");
    assert_eq!(sexp("a || b"), "(|| a b)");
}

#[test]
fn a_closure_can_be_called_immediately() {
    assert_eq!(sexp("(|x| x)(1)"), "(call (closure ((x _)) x) 1)");
}

// array literals

#[test]
fn parses_an_array_literal() {
    assert_eq!(sexp("[1, 2, 3]"), "(array 1 2 3)");
}

#[test]
fn parses_an_empty_array() {
    assert_eq!(sexp("[]"), "(array)");
}

#[test]
fn parses_an_array_with_a_trailing_comma() {
    assert_eq!(sexp("[1, 2,]"), "(array 1 2)");
}

#[test]
fn array_elements_are_expressions() {
    assert_eq!(sexp("[1 + 1, f(2)]"), "(array (+ 1 1) (call f 2))");
}

#[test]
fn arrays_nest() {
    assert_eq!(sexp("[[1, 2], [3]]"), "(array (array 1 2) (array 3))");
}

// `[` after an expression is still indexing
#[test]
fn indexing_still_parses_after_an_expression() {
    assert_eq!(sexp("a[0]"), "(index a 0)");
    assert_eq!(sexp("[1, 2][0]"), "(index (array 1 2) 0)");
}
