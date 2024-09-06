use std::ops::Range;

/// range of a token in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// byte offset of start position ( inclusive )
    pub start: usize,
    /// byte offset of end position ( exclusive )
    pub end: usize,
}

impl Span {
    /// create a span that covers nowhere
    pub fn new_none() -> Self {
        Self {
            start: usize::MAX,
            end: usize::MAX,
        }
    }
    /// check if the span covers nowhere
    pub fn is_none(&self) -> bool {
        self.start == usize::MAX
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    /// create a span that covers a single character
    pub fn new_single(point: usize) -> Self {
        Self {
            start: point,
            end: point + 1,
        }
    }
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }
    /// merge two spans into one.
    /// the new span will cover both of the original spans.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    /// merge two spans into one.
    /// the new span will cover both of the original spans.
    pub fn merge_ordered(&self, right: &Self) -> Self {
        Self {
            start: self.start,
            end: right.end,
        }
    }

    /// try to merge two spans into one. return None if either of the spans points to nowhere.
    pub fn try_merge(&self, other: &Self) -> Option<Self> {
        let merged = self.merge(other);
        if merged.end == usize::MAX {
            None
        } else {
            Some(merged)
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}
impl Into<Range<usize>> for Span {
    fn into(self) -> Range<usize> {
        self.start..self.end
    }
}
