//! parser scaffolding: a token cursor plus recoverable errors. expression
//! parsing lands in step 3.2+; item parsing in phase 4. for now
//! `parse_module` only really handles the empty-module case - anything past
//! `Eof` records a "not implemented yet" error and moves on, the same way
//! `nex-driver`'s unfinished subcommands do.

use crate::module::Module;
use crate::node::{NodeIdGen, NodeInfo};
use nex_lexer::{Span, Token, TokenKind};

#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        ParseError {
            message: message.into(),
            span,
        }
    }
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    ids: NodeIdGen,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            pos: 0,
            ids: NodeIdGen::new(),
            errors: Vec::new(),
        }
    }

    /// the token at the cursor. the token stream is expected to end with
    /// `Eof`; peek keeps returning it past the end rather than panicking.
    pub fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .expect("token stream must contain at least Eof")
    }

    pub fn at_eof(&self) -> bool {
        self.peek().is_eof()
    }

    /// consumes and returns the current token, then moves the cursor
    /// forward - except at `Eof`, which never advances past itself.
    pub fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if !tok.is_eof() {
            self.pos += 1;
        }
        tok
    }

    /// consumes the current token if it matches `kind`; otherwise records a
    /// `ParseError` and returns the (wrong) token anyway, so the caller can
    /// keep going instead of aborting the whole parse.
    pub fn expect(&mut self, kind: TokenKind) -> Token {
        let tok = self.peek().clone();
        if tok.kind == kind {
            self.advance()
        } else {
            self.errors.push(ParseError::new(
                format!(
                    "expected {}, found {}",
                    kind.describe(),
                    tok.kind.describe()
                ),
                tok.span,
            ));
            tok
        }
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// parses a whole source file into a `Module`.
    pub fn parse_module(&mut self) -> Module {
        let items = Vec::new();
        while !self.at_eof() {
            let tok = self.peek().clone();
            self.errors
                .push(ParseError::new("item parsing arrives in Phase 4", tok.span));
            self.advance();
        }
        let span = self.peek().span;
        Module::new(items, NodeInfo::new(self.ids.fresh(), span))
    }
}

/// tokenize-and-parse convenience wrapper.
pub fn parse_module(tokens: &[Token]) -> (Module, Vec<ParseError>) {
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module();
    (module, parser.errors)
}
