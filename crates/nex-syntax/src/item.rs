//! item ast: the top-level declarations a module is made of.

use crate::expr::Block;
use crate::node::{AstNode, HasSpan, Ident, NodeId, NodeInfo};
use crate::ty::Type;
use nex_lexer::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub info: NodeInfo,
    pub is_pub: bool,
    pub kind: ItemKind,
}

impl Item {
    /// private, which is the default for everything without `pub`
    pub fn new(kind: ItemKind, info: NodeInfo) -> Self {
        Item {
            info,
            is_pub: false,
            kind,
        }
    }

    pub fn new_pub(kind: ItemKind, info: NodeInfo) -> Self {
        Item {
            info,
            is_pub: true,
            kind,
        }
    }
}

impl HasSpan for Item {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Item {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    Fn(Fn),
    Struct(Struct),
    Enum(Enum),
    Use(Use),
    Mod(Mod),
    Impl(Impl),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub info: NodeInfo,
    pub name: Ident,
    pub ty: Type,
}

impl Param {
    pub fn new(name: Ident, ty: Type, info: NodeInfo) -> Self {
        Param { info, name, ty }
    }
}

impl HasSpan for Param {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Param {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fn {
    pub info: NodeInfo,
    pub name: Ident,
    pub generics: Vec<Ident>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

impl Fn {
    pub fn new(
        name: Ident,
        generics: Vec<Ident>,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Block,
        info: NodeInfo,
    ) -> Self {
        Fn {
            info,
            name,
            generics,
            params,
            return_type,
            body,
        }
    }
}

impl HasSpan for Fn {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Fn {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldDef {
    pub info: NodeInfo,
    pub name: Ident,
    pub ty: Type,
}

impl FieldDef {
    pub fn new(name: Ident, ty: Type, info: NodeInfo) -> Self {
        FieldDef { info, name, ty }
    }
}

impl HasSpan for FieldDef {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for FieldDef {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Struct {
    pub info: NodeInfo,
    pub name: Ident,
    pub generics: Vec<Ident>,
    pub fields: Vec<FieldDef>,
}

impl Struct {
    pub fn new(name: Ident, generics: Vec<Ident>, fields: Vec<FieldDef>, info: NodeInfo) -> Self {
        Struct {
            info,
            name,
            generics,
            fields,
        }
    }
}

impl HasSpan for Struct {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Struct {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// a variant's payload types, e.g. `T` in `Some(T)`; empty for a unit variant
/// like `None`
#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    pub info: NodeInfo,
    pub name: Ident,
    pub fields: Vec<Type>,
}

impl Variant {
    pub fn new(name: Ident, fields: Vec<Type>, info: NodeInfo) -> Self {
        Variant { info, name, fields }
    }
}

impl HasSpan for Variant {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Variant {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enum {
    pub info: NodeInfo,
    pub name: Ident,
    pub generics: Vec<Ident>,
    pub variants: Vec<Variant>,
}

impl Enum {
    pub fn new(name: Ident, generics: Vec<Ident>, variants: Vec<Variant>, info: NodeInfo) -> Self {
        Enum {
            info,
            name,
            generics,
            variants,
        }
    }
}

impl HasSpan for Enum {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Enum {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// `use a::b::c;` - no `as` aliasing or `{}` groups yet
#[derive(Clone, Debug, PartialEq)]
pub struct Use {
    pub info: NodeInfo,
    pub path: Vec<Ident>,
}

impl Use {
    pub fn new(path: Vec<Ident>, info: NodeInfo) -> Self {
        Use { info, path }
    }
}

impl HasSpan for Use {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Use {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// `mod foo;` (external file, `items: None`) or `mod foo { ... }` (inline)
#[derive(Clone, Debug, PartialEq)]
pub struct Mod {
    pub info: NodeInfo,
    pub name: Ident,
    pub items: Option<Vec<Item>>,
}

impl Mod {
    pub fn new(name: Ident, items: Option<Vec<Item>>, info: NodeInfo) -> Self {
        Mod { info, name, items }
    }
}

impl HasSpan for Mod {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Mod {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// a stub: the target type only, no methods yet
#[derive(Clone, Debug, PartialEq)]
pub struct Impl {
    pub info: NodeInfo,
    pub target: Ident,
}

impl Impl {
    pub fn new(target: Ident, info: NodeInfo) -> Self {
        Impl { info, target }
    }
}

impl HasSpan for Impl {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Impl {
    fn id(&self) -> NodeId {
        self.info.id
    }
}
