//! the parser: a token cursor plus recoverable errors, with a pratt loop
//! for expressions. item parsing lands in phase 4 - for now `parse_module`
//! only really handles the empty-module case, and anything past `Eof`
//! records a "not implemented yet" error and moves on, the same way
//! `nex-driver`'s unfinished subcommands do.

use crate::expr::{BinaryOp, Block, Expr, ExprKind, UnaryOp};
use crate::module::Module;
use crate::node::{NodeIdGen, NodeInfo, Spanned};
use crate::stmt::{Stmt, StmtKind};
use nex_lexer::{Span, Token, TokenKind};

/// binding power of the prefix operators (`-x`, `!x`). higher than every
/// infix operator, so `-a * b` parses as `(-a) * b`.
const PREFIX_BP: u8 = 19;

/// `(op, left_bp, right_bp)` for an infix operator, or `None` if the token
/// doesn't start one. every operator here is left-associative, encoded as
/// `right_bp == left_bp + 1`, so `a - b - c` parses as `(a - b) - c`.
///
/// the ladder runs lowest-to-highest exactly as the spec orders it: `||`
/// binds loosest, arithmetic tightest. ranges (`..`, `..=`) sit *below*
/// `||` and arrive in step 3.9; postfix call/field/index bind tighter than
/// any prefix operator and arrive in step 3.5.
fn infix_binding_power(kind: &TokenKind) -> Option<(BinaryOp, u8, u8)> {
    use TokenKind as T;
    let (op, bp) = match kind {
        T::PipePipe => (BinaryOp::Or, 1),
        T::AmpAmp => (BinaryOp::And, 3),
        T::EqEq => (BinaryOp::Eq, 5),
        T::BangEq => (BinaryOp::Ne, 5),
        T::Lt => (BinaryOp::Lt, 5),
        T::LtEq => (BinaryOp::Le, 5),
        T::Gt => (BinaryOp::Gt, 5),
        T::GtEq => (BinaryOp::Ge, 5),
        T::Pipe => (BinaryOp::BitOr, 7),
        T::Caret => (BinaryOp::BitXor, 9),
        T::Amp => (BinaryOp::BitAnd, 11),
        T::Shl => (BinaryOp::Shl, 13),
        T::Shr => (BinaryOp::Shr, 13),
        T::Plus => (BinaryOp::Add, 15),
        T::Minus => (BinaryOp::Sub, 15),
        T::Star => (BinaryOp::Mul, 17),
        T::Slash => (BinaryOp::Div, 17),
        T::Percent => (BinaryOp::Rem, 17),
        _ => return None,
    };
    Some((op, bp, bp + 1))
}

fn prefix_op(kind: &TokenKind) -> Option<UnaryOp> {
    match kind {
        TokenKind::Minus => Some(UnaryOp::Neg),
        TokenKind::Bang => Some(UnaryOp::Not),
        _ => None,
    }
}

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

    /// parses a full expression, honouring operator precedence.
    pub fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0)
    }

    /// the pratt loop: parse a prefix expression, then keep folding in
    /// infix operators for as long as they bind tighter than `min_bp`.
    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_prefix();

        while let Some((op, left_bp, right_bp)) = infix_binding_power(&self.peek().kind) {
            if left_bp < min_bp {
                break;
            }
            let op_tok = self.advance();
            let rhs = self.parse_expr_bp(right_bp);
            let span = lhs.info.span.merge(rhs.info.span);
            lhs = self.leaf(
                ExprKind::Binary {
                    op: Spanned::new(op, op_tok.span),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }

        lhs
    }

    /// a prefix operator applied to an operand, or a leaf.
    fn parse_prefix(&mut self) -> Expr {
        if let Some(op) = prefix_op(&self.peek().kind) {
            let op_tok = self.advance();
            let operand = self.parse_expr_bp(PREFIX_BP);
            let span = op_tok.span.merge(operand.info.span);
            return self.leaf(
                ExprKind::Unary {
                    op: Spanned::new(op, op_tok.span),
                    operand: Box::new(operand),
                },
                span,
            );
        }
        self.parse_leaf()
    }

    /// a literal, an identifier, a parenthesized expression, or a block.
    fn parse_leaf(&mut self) -> Expr {
        match self.peek().kind {
            TokenKind::LParen => return self.parse_paren(),
            TokenKind::LBrace => {
                let block = self.parse_block();
                let span = block.info.span;
                return self.leaf(ExprKind::Block(block), span);
            }
            _ => {}
        }

        let tok = self.advance();
        match tok.kind {
            TokenKind::Int(v) => self.leaf(ExprKind::Int(v), tok.span),
            TokenKind::Float(v) => self.leaf(ExprKind::Float(v), tok.span),
            TokenKind::Str(s) => self.leaf(ExprKind::Str(s), tok.span),
            TokenKind::True => self.leaf(ExprKind::Bool(true), tok.span),
            TokenKind::False => self.leaf(ExprKind::Bool(false), tok.span),
            TokenKind::Ident(name) => {
                self.leaf(ExprKind::Ident(Spanned::new(name, tok.span)), tok.span)
            }
            _ => {
                self.errors.push(ParseError::new(
                    format!("expected an expression, found {}", tok.kind.describe()),
                    tok.span,
                ));
                // NodeId::DUMMY marks this as synthesised during recovery,
                // not a real `()` value the source wrote.
                Expr::new(ExprKind::Unit, NodeInfo::dummy(tok.span))
            }
        }
    }

    /// `(expr)` - the parens only group, they leave no node of their own.
    /// `()` is the unit literal.
    fn parse_paren(&mut self) -> Expr {
        let open = self.expect(TokenKind::LParen);

        if self.peek().kind == TokenKind::RParen {
            let close = self.advance();
            return self.leaf(ExprKind::Unit, open.span.merge(close.span));
        }

        let inner = self.parse_expr();
        let close = self.expect(TokenKind::RParen);
        // the grouped expression keeps its own node and NodeId; only its
        // span widens to cover the parens, so diagnostics can point at the
        // whole `( ... )` rather than just the inside
        let span = open.span.merge(close.span);
        self.leaf(inner.kind, span)
    }

    /// `{ a; b; c }` - a braced sequence of statements. only expression
    /// statements exist so far; `let`/`return`/loops arrive in phase 4.
    fn parse_block(&mut self) -> Block {
        let open = self.expect(TokenKind::LBrace);
        let mut stmts = Vec::new();

        loop {
            match self.peek().kind {
                TokenKind::RBrace => break,
                // an unterminated block would otherwise spin forever here:
                // `advance` is a no-op at Eof, so `parse_expr` would keep
                // returning a dummy without consuming anything. bail out
                // through `expect` so the message matches the one a
                // half-parsed block (`{ a; b`) produces below.
                TokenKind::Eof => break,
                _ => {}
            }

            let expr = self.parse_expr();
            let span = expr.info.span;
            stmts.push(Stmt::new(
                StmtKind::Expr(expr),
                NodeInfo::new(self.ids.fresh(), span),
            ));

            // a trailing `;` is optional before `}`; the block's value is
            // its last expression statement either way
            if self.peek().kind == TokenKind::Semi {
                self.advance();
            } else {
                break;
            }
        }

        let close = self.expect(TokenKind::RBrace);
        let span = open.span.merge(close.span);
        Block::new(stmts, NodeInfo::new(self.ids.fresh(), span))
    }

    fn leaf(&mut self, kind: ExprKind, span: Span) -> Expr {
        Expr::new(kind, NodeInfo::new(self.ids.fresh(), span))
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

/// parses a single leaf expression (literal or identifier) from a token
/// stream. mainly useful for tests until step 3.3+ gives `Parser` a real
/// top-level expression entry point.
pub fn parse_expr(tokens: &[Token]) -> (Expr, Vec<ParseError>) {
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr();
    (expr, parser.errors)
}
