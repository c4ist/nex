//! Snapshot tests over the example programs. Regenerate with:
//!
//! ```text
//! INSTA_UPDATE=always cargo test -p nex-lexer
//! ```

use nex_lexer::dump_tokens;

#[test]
fn hello_world_token_stream() {
    let src = include_str!("../../../examples/hello.nex");
    insta::assert_snapshot!("hello", dump_tokens(src));
}

#[test]
fn tour_token_stream() {
    let src = include_str!("../../../examples/tour.nex");
    insta::assert_snapshot!("tour", dump_tokens(src));
}

#[test]
fn errors_are_included_in_the_dump() {
    let src = "let x = 1.foo; @ \"oops";
    insta::assert_snapshot!("errors", dump_tokens(src));
}
