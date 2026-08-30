//! recursive-descent parser with a pratt loop for expressions.
//!
//! errors are collected rather than thrown: every parse returns a tree, and
//! anything that went wrong is in `errors()`.

use crate::expr::{BinaryOp, Block, Expr, ExprKind, FieldInit, MatchArm, UnaryOp};
use crate::item::{Enum, FieldDef, Fn, Item, ItemKind, Mod, Param, Struct, Use, Variant};
use crate::module::Module;
use crate::node::{Ident, NodeIdGen, NodeInfo, Spanned};
use crate::pattern::{FieldPattern, Pattern, PatternKind};
use crate::stmt::{Stmt, StmtKind};
use crate::ty::{Type, TypeKind};
use nex_lexer::{Span, Token, TokenKind};

/// binds tighter than every infix operator, so `-a * b` is `(-a) * b`
const PREFIX_BP: u8 = 19;

/// binds looser than every infix operator
const RANGE_BP: u8 = 0;

/// `(op, left_bp, right_bp)`, lowest precedence first. all left-associative,
/// which is what `right_bp = left_bp + 1` encodes.
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

/// `Some(None)` for a plain `=`, `Some(Some(op))` for a compound assignment,
/// `None` when the token isn't an assignment at all.
fn assign_op(kind: &TokenKind) -> Option<Option<BinaryOp>> {
    match kind {
        TokenKind::Eq => Some(None),
        TokenKind::PlusEq => Some(Some(BinaryOp::Add)),
        TokenKind::MinusEq => Some(Some(BinaryOp::Sub)),
        TokenKind::StarEq => Some(Some(BinaryOp::Mul)),
        TokenKind::SlashEq => Some(Some(BinaryOp::Div)),
        _ => None,
    }
}

/// whether a statement has to be followed by `;` to be a statement rather
/// than the block's trailing value.
///
/// anything ending in a braced body doesn't: `while a { } b` is two
/// statements, not a missing semicolon.
fn needs_semi(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::While { .. } | StmtKind::ForIn { .. } => false,
        StmtKind::Expr(e) => !matches!(
            e.kind,
            ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Block(_)
        ),
        _ => true,
    }
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
    /// `if x { }` has to read as a condition plus a body, not as the struct
    /// literal `x { }`. set while parsing a condition or match scrutinee.
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

    pub fn peek(&self) -> &Token {
        self.peek_nth(0)
    }

    fn peek_nth(&self, n: usize) -> &Token {
        self.tokens
            .get(self.pos + n)
            .or_else(|| self.tokens.last())
            .expect("token stream must contain at least Eof")
    }

    pub fn at_eof(&self) -> bool {
        self.peek().is_eof()
    }

    /// note this is a no-op at `Eof`, so loops need their own `Eof` case or
    /// they will spin.
    pub fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if !tok.is_eof() {
            self.pos += 1;
        }
        tok
    }

    /// consumes the token if it matches, otherwise records an error and
    /// returns the token it found instead.
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

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    fn without_struct_literals<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = self.no_struct_literal;
        self.no_struct_literal = true;
        let out = f(self);
        self.no_struct_literal = saved;
        out
    }

    /// inside a delimiter the `if cond {` ambiguity is gone, so struct
    /// literals are allowed again.
    fn with_struct_literals<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = self.no_struct_literal;
        self.no_struct_literal = false;
        let out = f(self);
        self.no_struct_literal = saved;
        out
    }

    /// parses `element` repeatedly until `closer`, separated by commas, with
    /// a trailing comma allowed. does not consume the closer.
    ///
    /// stops at `Eof` too: `element` may not consume anything there, and the
    /// loop would spin.
    fn comma_separated<T>(
        &mut self,
        closer: TokenKind,
        mut element: impl FnMut(&mut Self) -> T,
    ) -> Vec<T> {
        let mut items = Vec::new();
        loop {
            if self.peek().kind == closer || self.at_eof() {
                break;
            }
            items.push(element(self));
            if self.peek().kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        items
    }

    /// skips to the next statement boundary after a bad statement, so one
    /// mistake doesn't produce an error for every token after it.
    fn synchronize(&mut self) {
        loop {
            match self.peek().kind {
                TokenKind::Eof | TokenKind::RBrace => return,
                TokenKind::Semi => {
                    self.advance();
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0)
    }

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

        // ranges are looser than everything above, so they fold in last.
        // parsing the end above RANGE_BP stops `a..b..c` from chaining.
        if min_bp == RANGE_BP && matches!(self.peek().kind, TokenKind::DotDot | TokenKind::DotDotEq)
        {
            let op_tok = self.advance();
            let inclusive = op_tok.kind == TokenKind::DotDotEq;
            let end = self.parse_expr_bp(RANGE_BP + 1);
            let span = lhs.info.span.merge(end.info.span);
            lhs = self.leaf(
                ExprKind::Range {
                    start: Box::new(lhs),
                    end: Box::new(end),
                    inclusive,
                },
                span,
            );
        }

        lhs
    }

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

    /// call, field access and indexing: `f(x)(y).z[0]`
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

    fn parse_call_args(&mut self) -> Vec<Expr> {
        self.comma_separated(TokenKind::RParen, |p| {
            p.with_struct_literals(|p| p.parse_expr())
        })
    }

    fn parse_leaf(&mut self) -> Expr {
        match self.peek().kind {
            TokenKind::LParen => return self.parse_paren(),
            TokenKind::LBrace => {
                let block = self.parse_block();
                let span = block.info.span;
                return self.leaf(ExprKind::Block(block), span);
            }
            TokenKind::If => return self.parse_if(),
            TokenKind::Match => return self.parse_match(),
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
                // the dummy id marks this as invented during recovery
                Expr::new(ExprKind::Unit, NodeInfo::dummy(tok.span))
            }
        }
    }

    /// parens only group; `()` is the unit literal
    fn parse_paren(&mut self) -> Expr {
        let open = self.expect(TokenKind::LParen);

        if self.peek().kind == TokenKind::RParen {
            let close = self.advance();
            return self.leaf(ExprKind::Unit, open.span.merge(close.span));
        }

        let inner = self.with_struct_literals(|p| p.parse_expr());
        let close = self.expect(TokenKind::RParen);
        self.leaf(inner.kind, open.span.merge(close.span))
    }

    fn parse_if(&mut self) -> Expr {
        let if_tok = self.expect(TokenKind::If);
        let cond = self.without_struct_literals(|p| p.parse_expr());

        let then = self.parse_block();
        let then_span = then.info.span;
        let then = self.leaf(ExprKind::Block(then), then_span);

        let mut span = if_tok.span.merge(then_span);
        let else_ = if self.peek().kind == TokenKind::Else {
            self.advance();
            // `else if` just nests another if in the else arm
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

    fn parse_match(&mut self) -> Expr {
        let match_tok = self.expect(TokenKind::Match);
        let scrutinee = self.without_struct_literals(|p| p.parse_expr());

        let open = self.expect(TokenKind::LBrace);
        if open.kind != TokenKind::LBrace {
            let span = match_tok.span.merge(open.span);
            return Expr::new(
                ExprKind::Match {
                    scrutinee: Box::new(scrutinee),
                    arms: Vec::new(),
                },
                NodeInfo::dummy(span),
            );
        }

        let arms = self.comma_separated(TokenKind::RBrace, |p| {
            let pattern = p.parse_pattern();
            p.expect(TokenKind::FatArrow);
            let body = p.with_struct_literals(|p| p.parse_expr());
            let span = pattern.info.span.merge(body.info.span);
            MatchArm::new(pattern, body, NodeInfo::new(p.ids.fresh(), span))
        });

        let close = self.expect(TokenKind::RBrace);
        let span = match_tok.span.merge(close.span);
        self.leaf(
            ExprKind::Match {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span,
        )
    }

    fn parse_pattern(&mut self) -> Pattern {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Int(v) => {
                self.advance();
                self.pattern(PatternKind::Int(v), tok.span)
            }
            TokenKind::Float(v) => {
                self.advance();
                self.pattern(PatternKind::Float(v), tok.span)
            }
            TokenKind::Str(ref s) => {
                let s = s.clone();
                self.advance();
                self.pattern(PatternKind::Str(s), tok.span)
            }
            TokenKind::True => {
                self.advance();
                self.pattern(PatternKind::Bool(true), tok.span)
            }
            TokenKind::False => {
                self.advance();
                self.pattern(PatternKind::Bool(false), tok.span)
            }
            // in a pattern the minus is part of the literal, not an operator
            TokenKind::Minus => {
                self.advance();
                let lit = self.peek().clone();
                match lit.kind {
                    TokenKind::Int(v) => {
                        self.advance();
                        self.pattern(PatternKind::Int(-v), tok.span.merge(lit.span))
                    }
                    TokenKind::Float(v) => {
                        self.advance();
                        self.pattern(PatternKind::Float(-v), tok.span.merge(lit.span))
                    }
                    _ => {
                        self.errors.push(ParseError::new(
                            format!(
                                "expected a number after `-` in a pattern, found {}",
                                lit.kind.describe()
                            ),
                            lit.span,
                        ));
                        Pattern::new(PatternKind::Wildcard, NodeInfo::dummy(tok.span))
                    }
                }
            }
            TokenKind::LParen => self.parse_tuple_pattern(),
            TokenKind::Ident(_) => self.parse_ident_pattern(),
            _ => {
                self.advance();
                self.errors.push(ParseError::new(
                    format!("expected a pattern, found {}", tok.kind.describe()),
                    tok.span,
                ));
                Pattern::new(PatternKind::Wildcard, NodeInfo::dummy(tok.span))
            }
        }
    }

    fn parse_tuple_pattern(&mut self) -> Pattern {
        let open = self.expect(TokenKind::LParen);
        let elems = self.comma_separated(TokenKind::RParen, |p| p.parse_pattern());
        let close = self.expect(TokenKind::RParen);
        self.pattern(PatternKind::Tuple(elems), open.span.merge(close.span))
    }

    /// a binding, `_`, `Enum::Variant(..)` or `Struct { .. }`, decided by
    /// what follows the name.
    fn parse_ident_pattern(&mut self) -> Pattern {
        let first = self.expect_ident();

        // `_` is an ordinary identifier as far as the lexer is concerned
        if first.value == "_" {
            return self.pattern(PatternKind::Wildcard, first.span);
        }

        let mut path = vec![first];
        while self.peek().kind == TokenKind::ColonColon {
            self.advance();
            path.push(self.expect_ident());
        }

        match self.peek().kind {
            TokenKind::LParen => {
                self.advance();
                let fields = self.comma_separated(TokenKind::RParen, |p| p.parse_pattern());
                let close = self.expect(TokenKind::RParen);
                let span = path[0].span.merge(close.span);
                self.pattern(PatternKind::EnumVariant { path, fields }, span)
            }
            TokenKind::LBrace if path.len() == 1 => {
                self.advance();
                let fields = self.comma_separated(TokenKind::RBrace, |p| {
                    let name = p.expect_ident();
                    p.expect(TokenKind::Colon);
                    let pat = p.parse_pattern();
                    let span = name.span.merge(pat.info.span);
                    FieldPattern::new(name, pat, NodeInfo::new(p.ids.fresh(), span))
                });
                let close = self.expect(TokenKind::RBrace);
                let name = path.remove(0);
                let span = name.span.merge(close.span);
                self.pattern(PatternKind::Struct { name, fields }, span)
            }
            // a bare name binds; a longer path is a unit variant
            _ => {
                if path.len() == 1 {
                    let name = path.remove(0);
                    let span = name.span;
                    self.pattern(PatternKind::Binding(name), span)
                } else {
                    let span = path[0].span.merge(path[path.len() - 1].span);
                    self.pattern(
                        PatternKind::EnumVariant {
                            path,
                            fields: Vec::new(),
                        },
                        span,
                    )
                }
            }
        }
    }

    fn parse_struct_literal(&mut self) -> Expr {
        let name = self.expect_ident();
        self.expect(TokenKind::LBrace);
        let fields = self.comma_separated(TokenKind::RBrace, |p| {
            let field_name = p.expect_ident();
            p.expect(TokenKind::Colon);
            let value = p.with_struct_literals(|p| p.parse_expr());
            let span = field_name.span.merge(value.info.span);
            FieldInit::new(field_name, value, NodeInfo::new(p.ids.fresh(), span))
        });

        let close = self.expect(TokenKind::RBrace);
        let span = name.span.merge(close.span);
        self.leaf(ExprKind::StructLit { name, fields }, span)
    }

    /// `let x = 1;`, `let mut x = 1;`, `let x: i32 = 1;`. the initializer is
    /// required; the type annotation isn't.
    fn parse_let(&mut self) -> Stmt {
        let let_tok = self.expect(TokenKind::Let);

        let mutable = self.peek().kind == TokenKind::Mut;
        if mutable {
            self.advance();
        }

        let name = self.expect_ident();

        let ty = if self.peek().kind == TokenKind::Colon {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };

        let eq = self.expect(TokenKind::Eq);
        // without the `=` there is no initializer to parse, and trying
        // anyway would report a second error for the same mistake
        let value = if eq.kind == TokenKind::Eq {
            self.with_struct_literals(|p| p.parse_expr())
        } else {
            Expr::new(ExprKind::Unit, NodeInfo::dummy(eq.span))
        };

        let span = let_tok.span.merge(value.info.span);
        self.stmt(
            StmtKind::Let {
                mutable,
                name,
                ty,
                value,
            },
            span,
        )
    }

    /// `return;` or `return expr;`
    fn parse_return(&mut self) -> Stmt {
        let ret = self.expect(TokenKind::Return);

        // nothing to return if the statement ends here, or if the block does
        if matches!(
            self.peek().kind,
            TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof
        ) {
            return self.stmt(StmtKind::Return(None), ret.span);
        }

        let value = self.with_struct_literals(|p| p.parse_expr());
        let span = ret.span.merge(value.info.span);
        self.stmt(StmtKind::Return(Some(value)), span)
    }

    fn stmt(&mut self, kind: StmtKind, span: Span) -> Stmt {
        Stmt::new(kind, NodeInfo::new(self.ids.fresh(), span))
    }

    /// `while cond { .. }`
    fn parse_while(&mut self) -> Stmt {
        let while_tok = self.expect(TokenKind::While);
        let cond = self.without_struct_literals(|p| p.parse_expr());
        let body = self.parse_block();
        let span = while_tok.span.merge(body.info.span);
        self.stmt(StmtKind::While { cond, body }, span)
    }

    /// `for x in 0..10 { .. }`. the binding is a plain name, not a pattern.
    fn parse_for(&mut self) -> Stmt {
        let for_tok = self.expect(TokenKind::For);
        let binding = self.expect_ident();
        self.expect(TokenKind::In);
        let iter = self.without_struct_literals(|p| p.parse_expr());
        let body = self.parse_block();
        let span = for_tok.span.merge(body.info.span);
        self.stmt(
            StmtKind::ForIn {
                binding,
                iter,
                body,
            },
            span,
        )
    }

    /// `i32`, `Option<T>`, `[T]`, `&T`, `fn(A, B) -> C`
    fn parse_type(&mut self) -> Type {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Amp => {
                self.advance();
                let inner = self.parse_type();
                let span = tok.span.merge(inner.info.span);
                self.ty(TypeKind::Ref(Box::new(inner)), span)
            }
            TokenKind::LBracket => {
                self.advance();
                let elem = self.parse_type();
                let close = self.expect(TokenKind::RBracket);
                let span = tok.span.merge(close.span);
                self.ty(TypeKind::Array(Box::new(elem)), span)
            }
            TokenKind::Fn => {
                self.advance();
                self.expect(TokenKind::LParen);
                let params = self.comma_separated(TokenKind::RParen, |p| p.parse_type());
                let close = self.expect(TokenKind::RParen);
                let mut span = tok.span.merge(close.span);
                let return_type = if self.peek().kind == TokenKind::Arrow {
                    self.advance();
                    let ret = self.parse_type();
                    span = span.merge(ret.info.span);
                    Some(Box::new(ret))
                } else {
                    None
                };
                self.ty(
                    TypeKind::Fn {
                        params,
                        return_type,
                    },
                    span,
                )
            }
            TokenKind::Ident(_) => {
                let name = self.expect_ident();
                let mut span = name.span;
                let mut args = Vec::new();
                if self.peek().kind == TokenKind::Lt {
                    self.advance();
                    args = self.comma_separated(TokenKind::Gt, |p| p.parse_type());
                    let close = self.expect(TokenKind::Gt);
                    span = span.merge(close.span);
                }
                self.ty(TypeKind::Named { name, args }, span)
            }
            _ => {
                // leave closers and separators alone: whatever contains this
                // type still needs them to find its own end. consuming a `}`
                // here would strip a struct of its closing brace.
                if !matches!(
                    tok.kind,
                    TokenKind::RBrace
                        | TokenKind::RParen
                        | TokenKind::RBracket
                        | TokenKind::Gt
                        | TokenKind::Comma
                        | TokenKind::Semi
                        | TokenKind::Eof
                ) {
                    self.advance();
                }
                self.errors.push(ParseError::new(
                    format!("expected a type, found {}", tok.kind.describe()),
                    tok.span,
                ));
                Type::new(
                    TypeKind::Named {
                        name: Spanned::new(String::new(), tok.span),
                        args: Vec::new(),
                    },
                    NodeInfo::dummy(tok.span),
                )
            }
        }
    }

    /// the `: Type` on a parameter or field. a missing `:` means no type
    /// follows, so asking for one would report the same mistake twice.
    fn parse_annotation(&mut self) -> Type {
        let colon = self.expect(TokenKind::Colon);
        if colon.kind == TokenKind::Colon {
            self.parse_type()
        } else {
            self.missing_type(colon.span)
        }
    }

    /// stand-in for a type the source never wrote; the error has already
    /// been reported by whatever noticed it was missing.
    fn missing_type(&mut self, span: Span) -> Type {
        Type::new(
            TypeKind::Named {
                name: Spanned::new(String::new(), span),
                args: Vec::new(),
            },
            NodeInfo::dummy(span),
        )
    }

    fn ty(&mut self, kind: TypeKind, span: Span) -> Type {
        Type::new(kind, NodeInfo::new(self.ids.fresh(), span))
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek().kind {
            TokenKind::Let => return self.parse_let(),
            TokenKind::Return => return self.parse_return(),
            TokenKind::While => return self.parse_while(),
            TokenKind::For => return self.parse_for(),
            TokenKind::Break => {
                let tok = self.advance();
                return self.stmt(StmtKind::Break, tok.span);
            }
            TokenKind::Continue => {
                let tok = self.advance();
                return self.stmt(StmtKind::Continue, tok.span);
            }
            _ => {}
        }

        // an assignment starts out looking like an expression, so parse one
        // and decide afterwards based on what follows it
        let expr = self.with_struct_literals(|p| p.parse_expr());

        if let Some(op) = assign_op(&self.peek().kind) {
            self.advance();
            let value = self.with_struct_literals(|p| p.parse_expr());
            let span = expr.info.span.merge(value.info.span);
            return self.stmt(
                StmtKind::Assign {
                    target: expr,
                    op,
                    value,
                },
                span,
            );
        }

        let span = expr.info.span;
        self.stmt(StmtKind::Expr(expr), span)
    }

    fn parse_block(&mut self) -> Block {
        let open = self.expect(TokenKind::LBrace);
        if open.kind != TokenKind::LBrace {
            // no block here at all, so don't also complain about a missing `}`
            return Block::new(Vec::new(), NodeInfo::dummy(open.span));
        }
        let mut stmts = Vec::new();

        loop {
            if matches!(self.peek().kind, TokenKind::RBrace | TokenKind::Eof) {
                break;
            }

            let errors_before = self.errors.len();
            let stmt = self.parse_stmt();
            let semi_required = needs_semi(&stmt);
            stmts.push(stmt);

            if self.errors.len() > errors_before {
                self.synchronize();
                continue;
            }

            if self.peek().kind == TokenKind::Semi {
                self.advance();
            } else if semi_required {
                // no semicolon and one was needed, so this was the block's
                // trailing value expression
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

    fn pattern(&mut self, kind: PatternKind, span: Span) -> Pattern {
        Pattern::new(kind, NodeInfo::new(self.ids.fresh(), span))
    }

    /// items aren't parsed yet, so this only really handles an empty file
    /// `fn name<T>(a: i32) -> i32 { .. }`
    fn parse_fn(&mut self) -> Item {
        let fn_tok = self.expect(TokenKind::Fn);
        let name = self.expect_ident();
        let generics = self.parse_generics();

        self.expect(TokenKind::LParen);
        let params = self.comma_separated(TokenKind::RParen, |p| {
            let param_name = p.expect_ident();
            let ty = p.parse_annotation();
            let span = param_name.span.merge(ty.info.span);
            Param::new(param_name, ty, NodeInfo::new(p.ids.fresh(), span))
        });
        self.expect(TokenKind::RParen);

        let return_type = if self.peek().kind == TokenKind::Arrow {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };

        let body = self.parse_block();
        let span = fn_tok.span.merge(body.info.span);
        let info = NodeInfo::new(self.ids.fresh(), span);
        self.item(
            ItemKind::Fn(Fn::new(name, generics, params, return_type, body, info)),
            span,
        )
    }

    /// `struct Point { x: f64, y: f64 }`. named fields only.
    fn parse_struct(&mut self) -> Item {
        let struct_tok = self.expect(TokenKind::Struct);
        let name = self.expect_ident();
        let generics = self.parse_generics();

        let open = self.expect(TokenKind::LBrace);
        if open.kind != TokenKind::LBrace {
            let span = struct_tok.span.merge(open.span);
            let info = NodeInfo::dummy(span);
            return Item::new(
                ItemKind::Struct(Struct::new(name, generics, Vec::new(), info)),
                info,
            );
        }

        let fields = self.comma_separated(TokenKind::RBrace, |p| {
            let field_name = p.expect_ident();
            let ty = p.parse_annotation();
            let span = field_name.span.merge(ty.info.span);
            FieldDef::new(field_name, ty, NodeInfo::new(p.ids.fresh(), span))
        });

        let close = self.expect(TokenKind::RBrace);
        let span = struct_tok.span.merge(close.span);
        let info = NodeInfo::new(self.ids.fresh(), span);
        self.item(
            ItemKind::Struct(Struct::new(name, generics, fields, info)),
            span,
        )
    }

    /// `enum Option<T> { Some(T), None }`. variants are unit or tuple;
    /// struct variants aren't in the language.
    fn parse_enum(&mut self) -> Item {
        let enum_tok = self.expect(TokenKind::Enum);
        let name = self.expect_ident();
        let generics = self.parse_generics();

        let open = self.expect(TokenKind::LBrace);
        if open.kind != TokenKind::LBrace {
            let span = enum_tok.span.merge(open.span);
            let info = NodeInfo::dummy(span);
            return Item::new(
                ItemKind::Enum(Enum::new(name, generics, Vec::new(), info)),
                info,
            );
        }

        let variants = self.comma_separated(TokenKind::RBrace, |p| {
            let variant_name = p.expect_ident();
            let mut span = variant_name.span;
            let mut payload = Vec::new();

            if p.peek().kind == TokenKind::LParen {
                p.advance();
                payload = p.comma_separated(TokenKind::RParen, |p| p.parse_type());
                let close = p.expect(TokenKind::RParen);
                span = span.merge(close.span);
            }

            Variant::new(variant_name, payload, NodeInfo::new(p.ids.fresh(), span))
        });

        let close = self.expect(TokenKind::RBrace);
        let span = enum_tok.span.merge(close.span);
        let info = NodeInfo::new(self.ids.fresh(), span);
        self.item(
            ItemKind::Enum(Enum::new(name, generics, variants, info)),
            span,
        )
    }

    /// `use a::b::c;`
    fn parse_use(&mut self) -> Item {
        let use_tok = self.expect(TokenKind::Use);

        let mut path = vec![self.expect_ident()];
        while self.peek().kind == TokenKind::ColonColon {
            self.advance();
            path.push(self.expect_ident());
        }

        let semi = self.expect(TokenKind::Semi);
        let span = use_tok.span.merge(semi.span);
        let info = NodeInfo::new(self.ids.fresh(), span);
        self.item(ItemKind::Use(Use::new(path, info)), span)
    }

    /// `mod foo;` for a separate file, or `mod foo { .. }` inline.
    fn parse_mod(&mut self) -> Item {
        let mod_tok = self.expect(TokenKind::Mod);
        let name = self.expect_ident();

        if self.peek().kind == TokenKind::LBrace {
            self.advance();
            let mut items = Vec::new();
            while !matches!(self.peek().kind, TokenKind::RBrace | TokenKind::Eof) {
                match self.parse_item() {
                    Some(item) => items.push(item),
                    None => {
                        self.advance();
                    }
                }
            }
            let close = self.expect(TokenKind::RBrace);
            let span = mod_tok.span.merge(close.span);
            let info = NodeInfo::new(self.ids.fresh(), span);
            return self.item(ItemKind::Mod(Mod::new(name, Some(items), info)), span);
        }

        let semi = self.expect(TokenKind::Semi);
        let span = mod_tok.span.merge(semi.span);
        let info = NodeInfo::new(self.ids.fresh(), span);
        self.item(ItemKind::Mod(Mod::new(name, None, info)), span)
    }

    /// one item, or `None` if the current token can't start one. reports
    /// nothing: the caller decides whether that's an error or a loop end.
    fn parse_item(&mut self) -> Option<Item> {
        let is_pub = self.peek().kind == TokenKind::Pub;
        if is_pub {
            self.advance();
        }

        let item = match self.peek().kind {
            TokenKind::Fn => self.parse_fn(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Use => self.parse_use(),
            TokenKind::Mod => self.parse_mod(),
            _ => {
                if is_pub {
                    let tok = self.peek().clone();
                    self.errors.push(ParseError::new(
                        format!(
                            "expected an item after `pub`, found {}",
                            tok.kind.describe()
                        ),
                        tok.span,
                    ));
                }
                return None;
            }
        };

        Some(if is_pub {
            Item::new_pub(item.kind, item.info)
        } else {
            item
        })
    }

    /// `<T, U>`, or nothing. bare names only; nex has no bounds.
    fn parse_generics(&mut self) -> Vec<Ident> {
        if self.peek().kind != TokenKind::Lt {
            return Vec::new();
        }
        self.advance();

        let generics = self.comma_separated(TokenKind::Gt, |p| p.expect_ident());
        self.expect(TokenKind::Gt);
        generics
    }

    fn item(&mut self, kind: ItemKind, span: Span) -> Item {
        Item::new(kind, NodeInfo::new(self.ids.fresh(), span))
    }

    pub fn parse_module(&mut self) -> Module {
        let mut items = Vec::new();
        while !self.at_eof() {
            match self.parse_item() {
                Some(item) => items.push(item),
                None => {
                    let tok = self.peek().clone();
                    self.errors.push(ParseError::new(
                        format!("expected an item, found {}", tok.kind.describe()),
                        tok.span,
                    ));
                    self.advance();
                }
            }
        }
        let span = self.peek().span;
        Module::new(items, NodeInfo::new(self.ids.fresh(), span))
    }
}

pub fn parse_module(tokens: &[Token]) -> (Module, Vec<ParseError>) {
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module();
    (module, parser.errors)
}

pub fn parse_expr(tokens: &[Token]) -> (Expr, Vec<ParseError>) {
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr();
    (expr, parser.errors)
}
