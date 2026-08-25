//! parser scaffolding tests: the token cursor and the empty-module case.
//! real item/expression parsing arrives in later steps.

use nex_lexer::{Token, TokenKind};
use nex_syntax::{parse_module, ParseError, Parser};

fn eof_only() -> Vec<Token> {
    let (tokens, errors) = nex_lexer::tokenize("");
    assert!(errors.is_empty(), "empty source should not lex with errors");
    tokens
}

#[test]
fn parses_an_empty_file_to_an_empty_module() {
    let tokens = eof_only();
    let (module, errors) = parse_module(&tokens);
    assert_eq!(module.items, Vec::new());
    assert_eq!(errors, Vec::new());
}

#[test]
fn peek_never_runs_past_eof() {
    let tokens = eof_only();
    let mut parser = Parser::new(&tokens);
    assert!(parser.at_eof());
    // advancing past Eof is a no-op; peek keeps returning it
    parser.advance();
    parser.advance();
    assert!(parser.at_eof());
    assert_eq!(parser.peek().kind, TokenKind::Eof);
}

#[test]
fn advance_returns_each_token_in_order_and_stops_at_eof() {
    let (tokens, errors) = nex_lexer::tokenize("fn x");
    assert!(errors.is_empty());
    let mut parser = Parser::new(&tokens);

    assert!(!parser.at_eof());
    let first = parser.advance();
    assert_eq!(first.kind, TokenKind::Fn);

    assert!(!parser.at_eof());
    let second = parser.advance();
    assert!(matches!(second.kind, TokenKind::Ident(_)));

    // both real tokens are consumed; the cursor now sits on Eof
    assert!(parser.at_eof());
    let third = parser.advance();
    assert_eq!(third.kind, TokenKind::Eof);
    assert!(parser.at_eof());
}

#[test]
fn expect_consumes_a_matching_token_without_an_error() {
    let (tokens, _) = nex_lexer::tokenize("fn");
    let mut parser = Parser::new(&tokens);
    let tok = parser.expect(TokenKind::Fn);
    assert_eq!(tok.kind, TokenKind::Fn);
    assert_eq!(parser.errors(), &[]);
}

#[test]
fn expect_records_an_error_on_a_mismatched_token_but_still_returns_it() {
    let (tokens, _) = nex_lexer::tokenize("fn");
    let mut parser = Parser::new(&tokens);
    let tok = parser.expect(TokenKind::Let);
    // still the actual token at the cursor, not consumed as the wrong kind
    assert_eq!(tok.kind, TokenKind::Fn);
    assert_eq!(parser.errors().len(), 1);
    assert_eq!(parser.errors()[0].span, tok.span);
}

// item parsing isn't implemented yet (phase 4); a non-empty file reports
// one "not implemented" error per leftover token instead of silently
// dropping them, and consumes everything so it terminates.
#[test]
fn parse_module_reports_every_leftover_token_for_non_empty_input() {
    let (tokens, lex_errors) = nex_lexer::tokenize("fn main() {}");
    assert!(lex_errors.is_empty());
    let non_eof_token_count = tokens.iter().filter(|t| !t.is_eof()).count();

    let (module, errors) = parse_module(&tokens);
    assert_eq!(module.items, Vec::new());
    assert_eq!(errors.len(), non_eof_token_count);
    for err in &errors {
        let ParseError { message, .. } = err;
        assert_eq!(message, "item parsing arrives in Phase 4");
    }
}
