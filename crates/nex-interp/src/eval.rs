//! expression evaluation.

use std::rc::Rc;

use nex_lexer::Span;
use nex_syntax::{BinaryOp, Block, Expr, ExprKind, Stmt, StmtKind, UnaryOp};

use crate::env::Scope;
use crate::error::{Result, RuntimeError};
use crate::flow::Flow;
use crate::value::Value;

pub struct Interpreter {
    globals: Rc<Scope>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: Scope::global(),
        }
    }

    pub fn globals(&self) -> &Rc<Scope> {
        &self.globals
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        let scope = Rc::clone(&self.globals);
        Ok(self.eval_in(expr, &scope)?.value())
    }

    pub fn eval_in(&mut self, expr: &Expr, scope: &Rc<Scope>) -> Result<Flow> {
        match &expr.kind {
            ExprKind::Int(v) => Ok(Flow::Normal(Value::Int(*v))),
            ExprKind::Float(v) => Ok(Flow::Normal(Value::Float(*v))),
            ExprKind::Bool(v) => Ok(Flow::Normal(Value::Bool(*v))),
            ExprKind::Str(s) => Ok(Flow::Normal(Value::str(s))),
            ExprKind::Unit => Ok(Flow::Normal(Value::Unit)),

            ExprKind::Ident(name) => scope.lookup(&name.value).map(Flow::Normal).ok_or_else(|| {
                RuntimeError::new(format!("`{}` is not defined", name.value), name.span)
            }),

            ExprKind::Unary { op, operand } => {
                let flow = self.eval_in(operand, scope)?;
                match flow {
                    Flow::Normal(value) => unary(op.value, value, expr.info.span).map(Flow::Normal),
                    jump => Ok(jump),
                }
            }

            ExprKind::Binary { op, lhs, rhs } => self.binary_expr(expr, op.value, lhs, rhs, scope),

            ExprKind::Block(block) => self.exec_block(block, scope),

            ExprKind::If { .. } => self.eval_if(expr, scope),

            other => Err(RuntimeError::new(
                format!("{} is not supported yet", describe(other)),
                expr.info.span,
            )),
        }
    }

    fn binary_expr(
        &mut self,
        expr: &Expr,
        op: BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        scope: &Rc<Scope>,
    ) -> Result<Flow> {
        // `&&` and `||` short-circuit, so the right side only runs when the
        // left hasn't already decided the answer
        if matches!(op, BinaryOp::And | BinaryOp::Or) {
            let left = match self.eval_in(lhs, scope)? {
                Flow::Normal(value) => as_bool(&value, lhs.info.span)?,
                jump => return Ok(jump),
            };
            let decided = match op {
                BinaryOp::And => !left,
                _ => left,
            };
            if decided {
                return Ok(Flow::Normal(Value::Bool(left)));
            }
            let right = match self.eval_in(rhs, scope)? {
                Flow::Normal(value) => as_bool(&value, rhs.info.span)?,
                jump => return Ok(jump),
            };
            return Ok(Flow::Normal(Value::Bool(right)));
        }

        let left = match self.eval_in(lhs, scope)? {
            Flow::Normal(value) => value,
            jump => return Ok(jump),
        };
        let right = match self.eval_in(rhs, scope)? {
            Flow::Normal(value) => value,
            jump => return Ok(jump),
        };
        binary(op, left, right, expr.info.span).map(Flow::Normal)
    }
}

impl Interpreter {
    /// runs a block in its own scope, so bindings inside it don't leak.
    /// the block's value is its last expression statement, matching what
    /// the parser documents.
    pub fn exec_block(&mut self, block: &Block, scope: &Rc<Scope>) -> Result<Flow> {
        let inner = Scope::child(scope);
        let mut value = Value::Unit;

        for stmt in &block.stmts {
            let flow = self.exec_stmt(stmt, &inner)?;
            if flow.is_jump() {
                return Ok(flow);
            }
            value = flow.value();
        }

        Ok(Flow::Normal(value))
    }

    pub fn exec_stmt(&mut self, stmt: &Stmt, scope: &Rc<Scope>) -> Result<Flow> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                // an if or a block in statement position can itself break
                // or return, so it needs the flow-aware path
                match &expr.kind {
                    ExprKind::Block(block) => self.exec_block(block, scope),
                    ExprKind::If { .. } => self.eval_if(expr, scope),
                    _ => self.eval_in(expr, scope),
                }
            }

            StmtKind::Let { name, value, .. } => {
                let value = match self.eval_in(value, scope)? {
                    Flow::Normal(value) => value,
                    jump => return Ok(jump),
                };
                scope.define(name.value.clone(), value);
                Ok(Flow::Normal(Value::Unit))
            }

            StmtKind::Assign { target, op, value } => self.exec_assign(target, *op, value, scope),

            StmtKind::While { cond, body } => {
                loop {
                    let test = match self.eval_in(cond, scope)? {
                        Flow::Normal(value) => as_bool(&value, cond.info.span)?,
                        jump => return Ok(jump),
                    };
                    if !test {
                        break;
                    }
                    match self.exec_block(body, scope)? {
                        // `continue` ends this iteration, not the loop
                        Flow::Normal(_) | Flow::Continue => {}
                        Flow::Break => break,
                        // a return has to keep going past the loop
                        ret @ Flow::Return(_) => return Ok(ret),
                    }
                }
                Ok(Flow::Normal(Value::Unit))
            }

            StmtKind::Break => Ok(Flow::Break),
            StmtKind::Continue => Ok(Flow::Continue),

            StmtKind::Return(value) => {
                let value = match value {
                    Some(expr) => match self.eval_in(expr, scope)? {
                        Flow::Normal(value) => value,
                        jump => return Ok(jump),
                    },
                    None => Value::Unit,
                };
                Ok(Flow::Return(value))
            }

            StmtKind::ForIn { .. } => Err(RuntimeError::new(
                "for loops are not supported yet",
                stmt.info.span,
            )),
        }
    }

    fn eval_if(&mut self, expr: &Expr, scope: &Rc<Scope>) -> Result<Flow> {
        let ExprKind::If { cond, then, else_ } = &expr.kind else {
            unreachable!("only called for an if expression");
        };

        let test = match self.eval_in(cond, scope)? {
            Flow::Normal(value) => as_bool(&value, cond.info.span)?,
            jump => return Ok(jump),
        };
        let branch = if test { Some(then) } else { else_.as_ref() };

        match branch {
            // both arms are blocks, and an `else if` is another if
            Some(branch) => match &branch.kind {
                ExprKind::Block(block) => self.exec_block(block, scope),
                ExprKind::If { .. } => self.eval_if(branch, scope),
                _ => self.eval_in(branch, scope),
            },
            // an if with no else is unit when the condition is false
            None => Ok(Flow::Normal(Value::Unit)),
        }
    }

    fn exec_assign(
        &mut self,
        target: &Expr,
        op: Option<BinaryOp>,
        value: &Expr,
        scope: &Rc<Scope>,
    ) -> Result<Flow> {
        let ExprKind::Ident(name) = &target.kind else {
            return Err(RuntimeError::new(
                "only variables can be assigned to for now",
                target.info.span,
            ));
        };

        let new = match self.eval_in(value, scope)? {
            Flow::Normal(value) => value,
            jump => return Ok(jump),
        };
        let new = match op {
            // `x += 1` reads x, applies the operator, writes back
            Some(op) => {
                let current = scope.lookup(&name.value).ok_or_else(|| {
                    RuntimeError::new(format!("`{}` is not defined", name.value), name.span)
                })?;
                binary(op, current, new, target.info.span)?
            }
            None => new,
        };

        if scope.assign(&name.value, new) {
            Ok(Flow::Normal(Value::Unit))
        } else {
            Err(RuntimeError::new(
                format!("`{}` is not defined", name.value),
                name.span,
            ))
        }
    }
}

fn describe(kind: &ExprKind) -> &'static str {
    match kind {
        ExprKind::Call { .. } => "calling a function",
        ExprKind::Field { .. } => "field access",
        ExprKind::Index { .. } => "indexing",
        ExprKind::StructLit { .. } => "a struct literal",
        ExprKind::If { .. } => "an if expression",
        ExprKind::Block(_) => "a block",
        ExprKind::Match { .. } => "a match expression",
        ExprKind::Range { .. } => "a range",
        _ => "this expression",
    }
}

fn as_bool(value: &Value, span: Span) -> Result<bool> {
    value.as_bool().ok_or_else(|| {
        RuntimeError::new(format!("expected bool, found {}", value.type_name()), span)
    })
}

fn unary(op: UnaryOp, value: Value, span: Span) -> Result<Value> {
    match (op, &value) {
        (UnaryOp::Neg, Value::Int(v)) => v
            .checked_neg()
            .map(Value::Int)
            .ok_or_else(|| RuntimeError::new("integer overflow", span)),
        (UnaryOp::Neg, Value::Float(v)) => Ok(Value::Float(-v)),
        (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
        _ => Err(RuntimeError::new(
            format!(
                "cannot apply `{}` to {}",
                match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                },
                value.type_name()
            ),
            span,
        )),
    }
}

fn binary(op: BinaryOp, left: Value, right: Value, span: Span) -> Result<Value> {
    use BinaryOp::*;

    // equality works on any two values, as long as they're the same type
    if matches!(op, Eq | Ne) {
        if left.type_name() != right.type_name() {
            return Err(unsupported(op, &left, &right, span));
        }
        let equal = left == right;
        return Ok(Value::Bool(if op == Eq { equal } else { !equal }));
    }

    match (&left, &right) {
        (Value::Int(a), Value::Int(b)) => int_op(op, *a, *b, span),
        (Value::Float(a), Value::Float(b)) => float_op(op, *a, *b, span),
        (Value::Str(a), Value::Str(b)) => match op {
            Add => Ok(Value::str(format!("{a}{b}"))),
            Lt => Ok(Value::Bool(a < b)),
            Le => Ok(Value::Bool(a <= b)),
            Gt => Ok(Value::Bool(a > b)),
            Ge => Ok(Value::Bool(a >= b)),
            _ => Err(unsupported(op, &left, &right, span)),
        },
        _ => Err(unsupported(op, &left, &right, span)),
    }
}

fn unsupported(op: BinaryOp, left: &Value, right: &Value, span: Span) -> RuntimeError {
    RuntimeError::new(
        format!(
            "cannot apply `{}` to {} and {}",
            op_str(op),
            left.type_name(),
            right.type_name()
        ),
        span,
    )
}

fn op_str(op: BinaryOp) -> &'static str {
    use BinaryOp::*;
    match op {
        Or => "||",
        And => "&&",
        Eq => "==",
        Ne => "!=",
        Lt => "<",
        Le => "<=",
        Gt => ">",
        Ge => ">=",
        BitOr => "|",
        BitXor => "^",
        BitAnd => "&",
        Shl => "<<",
        Shr => ">>",
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
    }
}

fn int_op(op: BinaryOp, a: i64, b: i64, span: Span) -> Result<Value> {
    use BinaryOp::*;

    // arithmetic is checked; wrapping silently is a worse default than an
    // error the programmer can see
    let checked = |v: Option<i64>| {
        v.map(Value::Int)
            .ok_or_else(|| RuntimeError::new("integer overflow", span))
    };

    match op {
        Add => checked(a.checked_add(b)),
        Sub => checked(a.checked_sub(b)),
        Mul => checked(a.checked_mul(b)),
        Div if b == 0 => Err(RuntimeError::new("division by zero", span)),
        Div => checked(a.checked_div(b)),
        Rem if b == 0 => Err(RuntimeError::new("remainder by zero", span)),
        Rem => checked(a.checked_rem(b)),
        BitAnd => Ok(Value::Int(a & b)),
        BitOr => Ok(Value::Int(a | b)),
        BitXor => Ok(Value::Int(a ^ b)),
        // shifting by more than the width is undefined in C and panics in
        // rust; here it's a plain runtime error
        Shl | Shr => {
            let bits = u32::try_from(b)
                .ok()
                .filter(|bits| *bits < i64::BITS)
                .ok_or_else(|| {
                    RuntimeError::new(format!("shift amount {b} is out of range"), span)
                })?;
            Ok(Value::Int(if op == Shl { a << bits } else { a >> bits }))
        }
        Lt => Ok(Value::Bool(a < b)),
        Le => Ok(Value::Bool(a <= b)),
        Gt => Ok(Value::Bool(a > b)),
        Ge => Ok(Value::Bool(a >= b)),
        Eq | Ne | And | Or => unreachable!("handled before dispatch"),
    }
}

fn float_op(op: BinaryOp, a: f64, b: f64, span: Span) -> Result<Value> {
    use BinaryOp::*;
    match op {
        Add => Ok(Value::Float(a + b)),
        Sub => Ok(Value::Float(a - b)),
        Mul => Ok(Value::Float(a * b)),
        Div => Ok(Value::Float(a / b)),
        Rem => Ok(Value::Float(a % b)),
        Lt => Ok(Value::Bool(a < b)),
        Le => Ok(Value::Bool(a <= b)),
        Gt => Ok(Value::Bool(a > b)),
        Ge => Ok(Value::Bool(a >= b)),
        BitAnd | BitOr | BitXor | Shl | Shr => Err(RuntimeError::new(
            format!("cannot apply `{}` to f64 and f64", op_str(op)),
            span,
        )),
        Eq | Ne | And | Or => unreachable!("handled before dispatch"),
    }
}
