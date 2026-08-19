use std::fmt;
use std::ops::Range;

/// half-open byte range `[start, end)` into a source file
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end, "span start must not exceed end");
        Span { start, end }
    }

    pub fn from_usize(start: usize, end: usize) -> Self {
        Span::new(start as u32, end as u32)
    }

    /// smallest span that covers both
    pub fn merge(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }

    pub fn len(self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub fn range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    /// the text this span covers. `src` must be the file it came from
    pub fn text(self, src: &str) -> &str {
        &src[self.range()]
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.range()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_covers_both() {
        let a = Span::new(2, 5);
        let b = Span::new(10, 12);
        assert_eq!(a.merge(b), Span::new(2, 12));
        assert_eq!(b.merge(a), Span::new(2, 12));
    }

    #[test]
    fn text_slices_source() {
        let src = "fn main";
        assert_eq!(Span::new(3, 7).text(src), "main");
    }

    #[test]
    fn len_and_empty() {
        assert_eq!(Span::new(4, 9).len(), 5);
        assert!(Span::new(4, 4).is_empty());
    }
}
