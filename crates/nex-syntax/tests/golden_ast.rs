//! whole-file AST snapshots for the example programs.

use nex_syntax::{parse_module, print_item};

fn parse_file(src: &str) -> String {
    let (tokens, lex_errors) = nex_lexer::tokenize(src);
    assert!(lex_errors.is_empty(), "example should lex cleanly");
    let (module, errors) = parse_module(&tokens);
    assert_eq!(
        errors.iter().map(|e| e.message.clone()).collect::<Vec<_>>(),
        Vec::<String>::new(),
        "example should parse cleanly"
    );
    module
        .items
        .iter()
        .map(print_item)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn hello_world_ast() {
    insta::assert_snapshot!(parse_file(include_str!("../../../examples/hello.nex")));
}
