//! a rough parser throughput baseline.
//!
//! not a benchmark harness: it exists so a bad regression shows up in the
//! ordinary test run. the numbers move with the machine, so the assertion
//! is deliberately loose and the printed figure is the useful part.

use std::time::Instant;

use nex_lexer::tokenize;
use nex_syntax::parse_module;

/// roughly 10k lines of representative nex
fn generate(functions: usize) -> String {
    let mut src = String::new();
    for i in 0..functions {
        src.push_str(&format!(
            "fn f{i}(a: i32, b: i32) -> i32 {{\n    \
             let x = a + b * 2;\n    \
             let mut total = 0;\n    \
             for j in 0..10 {{ total += x; }}\n    \
             while total > 0 {{ total -= 1; }}\n    \
             if total > x {{ return total; }} else {{ return x; }}\n\
             }}\n\n"
        ));
    }
    src
}

#[test]
fn parses_a_large_file_in_reasonable_time() {
    let src = generate(1_250);
    let lines = src.lines().count();
    assert!(lines >= 10_000, "expected ~10k lines, got {lines}");

    let start = Instant::now();
    let (tokens, lex_errors) = tokenize(&src);
    let lexed = start.elapsed();

    let start = Instant::now();
    let (module, errors) = parse_module(&tokens);
    let parsed = start.elapsed();

    assert!(lex_errors.is_empty(), "generated source should lex cleanly");
    assert!(errors.is_empty(), "generated source should parse cleanly");
    assert_eq!(module.items.len(), 1_250);

    let total = lexed + parsed;
    println!(
        "{lines} lines, {} tokens: lex {lexed:?}, parse {parsed:?}, total {total:?} \
         ({:.0} lines/sec)",
        tokens.len(),
        lines as f64 / total.as_secs_f64()
    );

    // debug builds are slow, so this only catches something pathological
    // like accidental quadratic behaviour
    assert!(
        total.as_secs() < 10,
        "parsing {lines} lines took {total:?}, which suggests a regression"
    );
}
