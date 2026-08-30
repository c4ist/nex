//! runtime values.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use nex_syntax::{Block, Ident, Param};

/// heap values are shared rather than copied, so `Clone` on a `Value` is
/// cheap and two bindings can refer to the same array or struct.
#[derive(Clone)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Rc<str>),
    Array(Rc<RefCell<Vec<Value>>>),
    Struct(Rc<StructValue>),
    Enum(Rc<EnumValue>),
    Fn(Rc<FnValue>),
    Builtin(Builtin),
}

pub struct StructValue {
    pub name: String,
    /// ordered so `Display` is stable regardless of how it was built
    pub fields: RefCell<BTreeMap<String, Value>>,
}

pub struct EnumValue {
    /// `Option::Some`, kept whole for display and comparison
    pub path: String,
    pub variant: String,
    pub payload: Vec<Value>,
}

pub struct FnValue {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
    /// generic parameter names, unused until there's a type checker
    pub generics: Vec<Ident>,
}

/// a function implemented in rust rather than nex
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Builtin {
    pub name: &'static str,
    /// `None` when the builtin takes any number of arguments
    pub arity: Option<usize>,
}

impl Value {
    pub fn str(s: impl AsRef<str>) -> Value {
        Value::Str(Rc::from(s.as_ref()))
    }

    pub fn array(items: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(items)))
    }

    /// the name used in error messages: "expected int, found str"
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "()",
            Value::Int(_) => "i32",
            Value::Float(_) => "f64",
            Value::Bool(_) => "bool",
            Value::Str(_) => "str",
            Value::Array(_) => "array",
            Value::Struct(_) => "struct",
            Value::Enum(_) => "enum",
            Value::Fn(_) | Value::Builtin(_) => "fn",
        }
    }

    /// only `bool` is truthy; there are no implicit conversions
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// what `print` writes: strings are bare, everything else reads like source
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => f.write_str("()"),
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => {
                // `1.0` rather than `1`, so floats stay distinguishable
                if v.fract() == 0.0 && v.is_finite() {
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => f.write_str(s),
            Value::Array(items) => {
                let items = items.borrow();
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str("]")
            }
            Value::Struct(s) => {
                let fields = s.fields.borrow();
                write!(f, "{} {{", s.name)?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, " {name}: {value}")?;
                }
                f.write_str(" }")
            }
            Value::Enum(e) => {
                write!(f, "{}::{}", e.path, e.variant)?;
                if e.payload.is_empty() {
                    return Ok(());
                }
                f.write_str("(")?;
                for (i, value) in e.payload.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{value}")?;
                }
                f.write_str(")")
            }
            Value::Fn(func) => write!(f, "fn {}", func.name),
            Value::Builtin(b) => write!(f, "fn {}", b.name),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // quoted, so a string is distinguishable from an identifier
            Value::Str(s) => write!(f, "{s:?}"),
            other => write!(f, "{other}"),
        }
    }
}

/// structural equality. functions compare by identity, since comparing
/// bodies would be surprising and comparing names would be wrong.
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Unit, Value::Unit) => true,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
            (Value::Struct(a), Value::Struct(b)) => {
                a.name == b.name && *a.fields.borrow() == *b.fields.borrow()
            }
            (Value::Enum(a), Value::Enum(b)) => {
                a.path == b.path && a.variant == b.variant && a.payload == b.payload
            }
            (Value::Fn(a), Value::Fn(b)) => Rc::ptr_eq(a, b),
            (Value::Builtin(a), Value::Builtin(b)) => a == b,
            _ => false,
        }
    }
}
