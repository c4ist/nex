//! identity and location plumbing shared by every ast node.
//!
//! two rules hold across the whole tree:
//!
//! 1. every node has a [`Span`], so later passes (type checker, lsp) can always
//!    point at the source text that caused something.
//! 2. every node has a unique [`NodeId`], so later passes can hang info off a
//!    node in side tables instead of mutating the tree. the ast is immutable
//!    once parsed.

use nex_lexer::Span;
use std::fmt;

/// a node's identity within one parsed module
///
/// ids are dense and handed out in order by [`NodeIdGen`], so they double as
/// indices into side tables (`Vec<T>` keyed by `id.index()`)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);

impl NodeId {
    /// placeholder for nodes we invent during error recovery, which don't
    /// correspond to anything the user actually wrote
    pub const DUMMY: NodeId = NodeId(u32::MAX);

    /// raw index. only [`NodeIdGen`] should be building ids directly
    pub fn as_u32(self) -> u32 {
        self.0
    }

    /// slot in a side table keyed by node id
    ///
    /// # panics
    ///
    /// on [`NodeId::DUMMY`], which has no slot
    pub fn index(self) -> usize {
        assert!(!self.is_dummy(), "NodeId::DUMMY has no side-table index");
        self.0 as usize
    }

    pub fn is_dummy(self) -> bool {
        self.0 == u32::MAX
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dummy() {
            f.write_str("#dummy")
        } else {
            write!(f, "#{}", self.0)
        }
    }
}

/// hands out fresh ids while parsing one module
#[derive(Debug, Default)]
pub struct NodeIdGen {
    next: u32,
}

impl NodeIdGen {
    pub fn new() -> Self {
        NodeIdGen::default()
    }

    /// next unused id
    ///
    /// # panics
    ///
    /// past `u32::MAX - 1` nodes, since that would collide with
    /// [`NodeId::DUMMY`]. not a file size worth handling gracefully.
    pub fn fresh(&mut self) -> NodeId {
        assert!(self.next < u32::MAX, "exhausted the NodeId space");
        let id = NodeId(self.next);
        self.next += 1;
        id
    }

    /// how many ids we've handed out, ie. the length a side table needs
    pub fn allocated(&self) -> usize {
        self.next as usize
    }
}

/// the identity + location every ast node embeds
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NodeInfo {
    pub id: NodeId,
    pub span: Span,
}

impl NodeInfo {
    pub fn new(id: NodeId, span: Span) -> Self {
        NodeInfo { id, span }
    }

    /// a node that isn't in the source, for error recovery
    pub fn dummy(span: Span) -> Self {
        NodeInfo {
            id: NodeId::DUMMY,
            span,
        }
    }
}

impl fmt::Debug for NodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.id, self.span)
    }
}

/// anything that knows where it came from
pub trait HasSpan {
    fn span(&self) -> Span;
}

/// a spanned thing that also has an identity
pub trait AstNode: HasSpan {
    fn id(&self) -> NodeId;

    fn info(&self) -> NodeInfo {
        NodeInfo::new(self.id(), self.span())
    }
}

impl HasSpan for NodeInfo {
    fn span(&self) -> Span {
        self.span
    }
}

impl AstNode for NodeInfo {
    fn id(&self) -> NodeId {
        self.id
    }
}

/// attaches a span to a value that doesn't need its own identity, like an
/// ident, a field name or an operator
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Spanned { value, span }
    }

    /// map the value, keep the span
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            value: &self.value,
            span: self.span,
        }
    }
}

impl<T> HasSpan for Spanned<T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.value, self.span)
    }
}

/// an identifier as written in the source
pub type Ident = Spanned<String>;

/// smallest span covering every element, or `fallback` if there are none
///
/// used to give a parent node a span derived from its children
pub fn spanning<T: HasSpan>(items: &[T], fallback: Span) -> Span {
    let mut iter = items.iter();
    match iter.next() {
        None => fallback,
        Some(first) => iter.fold(first.span(), |acc, item| acc.merge(item.span())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_sequential_and_dense() {
        let mut gen = NodeIdGen::new();
        let a = gen.fresh();
        let b = gen.fresh();
        let c = gen.fresh();
        assert_eq!([a.as_u32(), b.as_u32(), c.as_u32()], [0, 1, 2]);
        assert_eq!([a.index(), b.index(), c.index()], [0, 1, 2]);
        assert_eq!(gen.allocated(), 3);
    }

    #[test]
    fn ids_are_distinct() {
        let mut gen = NodeIdGen::new();
        let ids: Vec<NodeId> = (0..64).map(|_| gen.fresh()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len());
    }

    #[test]
    fn ids_index_a_side_table() {
        let mut gen = NodeIdGen::new();
        let ids: Vec<NodeId> = (0..4).map(|_| gen.fresh()).collect();
        let mut types = vec!["?"; gen.allocated()];
        types[ids[2].index()] = "i32";
        assert_eq!(types, vec!["?", "?", "i32", "?"]);
    }

    #[test]
    fn dummy_is_recognisable_and_never_allocated() {
        let mut gen = NodeIdGen::new();
        assert!(NodeId::DUMMY.is_dummy());
        assert!(!gen.fresh().is_dummy());
    }

    #[test]
    #[should_panic(expected = "no side-table index")]
    fn dummy_has_no_index() {
        let _ = NodeId::DUMMY.index();
    }

    #[test]
    fn debug_formats_are_compact() {
        let mut gen = NodeIdGen::new();
        let id = gen.fresh();
        assert_eq!(format!("{id:?}"), "#0");
        assert_eq!(format!("{:?}", NodeId::DUMMY), "#dummy");
        assert_eq!(
            format!("{:?}", NodeInfo::new(id, Span::new(3, 7))),
            "#0@3..7"
        );
    }

    #[test]
    fn node_info_implements_the_traits() {
        let info = NodeInfo::new(NodeId::DUMMY, Span::new(1, 2));
        assert_eq!(info.span(), Span::new(1, 2));
        assert_eq!(info.id(), NodeId::DUMMY);
        assert_eq!(info.info(), info);
    }

    #[test]
    fn spanned_maps_and_keeps_its_span() {
        let ident: Ident = Spanned::new("count".to_string(), Span::new(4, 9));
        assert_eq!(ident.span(), Span::new(4, 9));
        assert_eq!(format!("{ident:?}"), "\"count\"@4..9");

        let length = ident.as_ref().map(|s| s.len());
        assert_eq!(length.value, 5);
        assert_eq!(length.span, Span::new(4, 9));
    }

    #[test]
    fn spanning_covers_all_children() {
        let items = vec![
            Spanned::new((), Span::new(10, 12)),
            Spanned::new((), Span::new(4, 6)),
            Spanned::new((), Span::new(20, 25)),
        ];
        assert_eq!(spanning(&items, Span::new(0, 0)), Span::new(4, 25));
    }

    #[test]
    fn spanning_uses_the_fallback_when_empty() {
        let empty: Vec<Spanned<()>> = Vec::new();
        assert_eq!(spanning(&empty, Span::new(7, 8)), Span::new(7, 8));
    }
}
