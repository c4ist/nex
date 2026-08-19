//! The Nex lexer: turns source text into a stream of [`Token`]s.
//!
//! ```
//! use nex_lexer::{tokenize, TokenKind};
//!
//! let (tokens, errors) = tokenize("let x = 1;");
//! assert!(errors.is_empty());
//! assert_eq!(tokens[0].kind, TokenKind::Let);
//! ```

mod error;
mod lexer;
mod span;
mod token;

pub use error::{LexError, LexErrorKind};
pub use lexer::Lexer;
pub use span::Span;
pub use token::{Token, TokenKind};

/// Scans `src` completely, returning every token (ending in [`TokenKind::Eof`])
/// alongside every recoverable error encountered.
pub fn tokenize(src: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lexer = Lexer::new(src);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token() {
        tokens.push(token);
    }
    (tokens, lexer.into_errors())
}

/// Scans `src`, discarding spans. Convenient for tests that only care about
/// the shape of the token stream.
pub fn tokenize_kinds(src: &str) -> Vec<TokenKind> {
    tokenize(src).0.into_iter().map(|t| t.kind).collect()
}

/// Renders a token stream as one `Kind@start..end` line per token.
///
/// This is the canonical debug format used by snapshot tests and by
/// `nex lex`, so it is deliberately stable.
pub fn dump_tokens(src: &str) -> String {
    let (tokens, errors) = tokenize(src);
    let mut out = String::new();
    for token in &tokens {
        out.push_str(&format!("{:?}@{:?}\n", token.kind, token.span));
    }
    if !errors.is_empty() {
        out.push_str("\n-- errors --\n");
        for error in &errors {
            out.push_str(&format!("{}@{:?}\n", error.kind, error.span));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_only_eof() {
        let (tokens, errors) = tokenize("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        assert_eq!(tokens[0].span, Span::new(0, 0));
        assert!(errors.is_empty());
    }

    #[test]
    fn eof_is_produced_exactly_once() {
        let mut lexer = Lexer::new("x");
        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::Ident(_)
        ));
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eof);
        assert!(lexer.next_token().is_none());
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn lexer_is_an_iterator() {
        let kinds: Vec<_> = Lexer::new("1 + 2").map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Int(1),
                TokenKind::Plus,
                TokenKind::Int(2),
                TokenKind::Eof
            ]
        );
    }
}
