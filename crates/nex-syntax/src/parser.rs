//! the parser: a token cursor plus recoverable errors, with a pratt loop
//! for expressions. item parsing lands in phase 4 - for now `parse_module`
//! only really handles the empty-module case, and anything past `Eof`
//! records a "not implemented yet" error and moves on, the same way
//! `nex-driver`'s unfinished subcommands do.

use crate::expr::{BinaryOp, Block, Expr, ExprKind, FieldInit, UnaryOp};
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
    /// set while parsing the condition of an `if`/`while`/`for`, where a
    /// bare `Ident {` must read as "condition, then the body's opening
    /// brace" rather than as a struct literal. `if x { }` would otherwise
    /// parse `x { }` as a struct literal and then find no body.
    ///
    /// nothing sets this until `if` arrives in step 3.7; the flag and the
    /// `with_struct_literals` escape hatch land here because struct
    /// literals are what create the ambiguity in the first place.
    no_struct_literal: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            pos: 0,
            ids: NodeIdGen::new(),
            errors: Vec::new(),
            no_struct_literal: false,
        }
    }

    /// the token at the cursor. the token stream is expected to end with
    /// `Eof`; peek keeps returning it past the end rather than panicking.
    pub fn peek(&self) -> &Token {
        self.peek_nth(0)
    }

    /// the token `n` positions past the cursor, clamped to the trailing
    /// `Eof` so lookahead never runs off the end.
    fn peek_nth(&self, n: usize) -> &Token {
        self.tokens
            .get(self.pos + n)
            .or_else(|| self.tokens.last())
            .expect("token stream must contain at least Eof")
    }

    /// runs `f` with struct literals disabled, then restores the previous
    /// setting. saves-and-restores rather than clearing, so a nested
    /// condition (`if a { }` inside another condition) still behaves.
    fn without_struct_literals<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = self.no_struct_literal;
        self.no_struct_literal = true;
        let out = f(self);
        self.no_struct_literal = saved;
        out
    }

    /// re-enables struct literals inside a nested delimiter, where the
    /// `if cond {` ambiguity can't arise: `if f(P { x: 1 }) { }` is fine.
    fn with_struct_literals<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = self.no_struct_literal;
        self.no_struct_literal = false;
        let out = f(self);
        self.no_struct_literal = saved;
        out
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
        self.parse_postfix()
    }

    /// a leaf followed by any number of postfix chains: `f(x)(y).z[0]`.
    /// these bind tighter than the prefix operators, so `-a.b()` negates
    /// the *result* of the call.
    fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_leaf();

        loop {
            expr = match self.peek().kind {
                TokenKind::LParen => {
                    self.advance();
                    let args = self.parse_call_args();
                    let close = self.expect(TokenKind::RParen);
                    let span = expr.info.span.merge(close.span);
                    self.leaf(
                        ExprKind::Call {
                            callee: Box::new(expr),
                            args,
                        },
                        span,
                    )
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.expect_ident();
                    let span = expr.info.span.merge(name.span);
                    self.leaf(
                        ExprKind::Field {
                            base: Box::new(expr),
                            field: name,
                        },
                        span,
                    )
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.with_struct_literals(|p| p.parse_expr());
                    let close = self.expect(TokenKind::RBracket);
                    let span = expr.info.span.merge(close.span);
                    self.leaf(
                        ExprKind::Index {
                            base: Box::new(expr),
                            index: Box::new(index),
                        },
                        span,
                    )
                }
                _ => break,
            };
        }

        expr
    }

    /// a comma-separated argument list, already past the `(` and stopping
    /// before the `)`. a trailing comma is allowed.
    fn parse_call_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        loop {
            // `)` ends the list; Eof would otherwise spin, since neither
            // `parse_expr` nor `expect` consumes anything there
            if matches!(self.peek().kind, TokenKind::RParen | TokenKind::Eof) {
                break;
            }
            args.push(self.with_struct_literals(|p| p.parse_expr()));
            if self.peek().kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        args
    }

    /// consumes an identifier, or reports one and yields a placeholder so
    /// the caller can keep parsing.
    fn expect_ident(&mut self) -> Spanned<String> {
        let tok = self.peek().clone();
        if let TokenKind::Ident(name) = tok.kind {
            self.advance();
            Spanned::new(name, tok.span)
        } else {
            self.errors.push(ParseError::new(
                format!("expected identifier, found {}", tok.kind.describe()),
                tok.span,
            ));
            Spanned::new(String::new(), tok.span)
        }
    }

    /// a literal, an identifier, a struct literal, a parenthesized
    /// expression, or a block.
    fn parse_leaf(&mut self) -> Expr {
        match self.peek().kind {
            TokenKind::LParen => return self.parse_paren(),
            TokenKind::LBrace => {
                let block = self.parse_block();
                let span = block.info.span;
                return self.leaf(ExprKind::Block(block), span);
            }
            TokenKind::If => return self.parse_if(),
            TokenKind::Ident(_)
                if !self.no_struct_literal && self.peek_nth(1).kind == TokenKind::LBrace =>
            {
                return self.parse_struct_literal();
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

        let inner = self.with_struct_literals(|p| p.parse_expr());
        let close = self.expect(TokenKind::RParen);
        // the grouped expression keeps its own node and NodeId; only its
        // span widens to cover the parens, so diagnostics can point at the
        // whole `( ... )` rather than just the inside
        let span = open.span.merge(close.span);
        self.leaf(inner.kind, span)
    }

    /// `if cond { .. }`, `if cond { .. } else { .. }`, or an `else if`
    /// chain. both arms are blocks; `else` is optional.
    fn parse_if(&mut self) -> Expr {
        let if_tok = self.expect(TokenKind::If);

        // a bare `Ident {` in condition position is the condition followed
        // by the body's brace, never a struct literal
        let cond = self.without_struct_literals(|p| p.parse_expr());

        let then = self.parse_block();
        let then_span = then.info.span;
        let then = self.leaf(ExprKind::Block(then), then_span);

        let mut span = if_tok.span.merge(then_span);
        let else_ = if self.peek().kind == TokenKind::Else {
            self.advance();
            // `else if` chains by nesting another if-expression in the
            // else arm rather than by a dedicated node
            let branch = if self.peek().kind == TokenKind::If {
                self.parse_if()
            } else {
                let block = self.parse_block();
                let block_span = block.info.span;
                self.leaf(ExprKind::Block(block), block_span)
            };
            span = span.merge(branch.info.span);
            Some(Box::new(branch))
        } else {
            None
        };

        self.leaf(
            ExprKind::If {
                cond: Box::new(cond),
                then: Box::new(then),
                else_,
            },
            span,
        )
    }

    /// `Point { x: 1.0, y: 2.0 }`. only reached when struct literals are
    /// permitted here; see `no_struct_literal`.
    fn parse_struct_literal(&mut self) -> Expr {
        let name = self.expect_ident();
        self.expect(TokenKind::LBrace);
        let mut fields = Vec::new();

        loop {
            // `}` ends the list; Eof would otherwise spin, since nothing
            // below consumes a token there
            if matches!(self.peek().kind, TokenKind::RBrace | TokenKind::Eof) {
                break;
            }

            let field_name = self.expect_ident();
            self.expect(TokenKind::Colon);
            // inside the braces the ambiguity is gone, so a nested struct
            // literal is allowed even in an `if` condition
            let value = self.with_struct_literals(|p| p.parse_expr());
            let span = field_name.span.merge(value.info.span);
            fields.push(FieldInit::new(
                field_name,
                value,
                NodeInfo::new(self.ids.fresh(), span),
            ));

            if self.peek().kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }

        let close = self.expect(TokenKind::RBrace);
        let span = name.span.merge(close.span);
        self.leaf(ExprKind::StructLit { name, fields }, span)
    }

    /// `{ a; b; c }` - a braced sequence of statements. only expression
    /// statements exist so far; `let`/`return`/loops arrive in phase 4.
    fn parse_block(&mut self) -> Block {
        let open = self.expect(TokenKind::LBrace);
        if open.kind != TokenKind::LBrace {
            // there is no block here at all. returning early keeps one
            // real problem to one diagnostic - carrying on would also
            // report a missing `}` for a brace the source never opened.
            return Block::new(Vec::new(), NodeInfo::dummy(open.span));
        }
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

            let expr = self.with_struct_literals(|p| p.parse_expr());
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
