/// A read-only pointer into a slice, supporting sequential traversal with peek and rewind.
///
/// Used internally by the HTML tokenizer and parser to iterate over character
/// streams while supporting lookahead and backtracking.
pub(crate) struct VecPointerRef<'a, T> {
    values: &'a [T],
    /// The current index position within the slice.
    pub(crate) index: usize,
}

impl<'a, T> VecPointerRef<'a, T> {
    /// Create a new pointer starting at position 0.
    pub(crate) fn new(values: &[T]) -> VecPointerRef<'_, T> {
        VecPointerRef { values, index: 0 }
    }

    /// Returns a reference to the element at the current index, or `None` if past the end.
    pub(crate) fn current(&self) -> Option<&T> {
        self.get(self.index)
    }

    /// Returns the element at the current index and advances by one.
    pub(crate) fn next(&mut self) -> Option<&T> {
        self.next_add(1)
    }

    /// Returns the element at the current index and advances by `i` positions.
    pub(crate) fn next_add(&mut self, i: usize) -> Option<&T> {
        let index = self.index;
        self.index = self.index.saturating_add(i);
        self.get(index)
    }

    /// Moves the index back by one and returns the element at the new position.
    pub(crate) fn prev(&mut self) -> Option<&T> {
        self.prev_sub(1)
    }

    /// Moves the index back by `i` positions and returns the element at the new position.
    ///
    /// Returns `None` if `i` is greater than the current index (would underflow).
    pub(crate) fn prev_sub(&mut self, i: usize) -> Option<&T> {
        if i > self.index {
            return None;
        } else {
            self.index -= i;
        }
        self.current()
    }

    /// Returns a reference to the element `i` positions ahead without advancing.
    pub(crate) fn peek_add(&self, i: usize) -> Option<&T> {
        self.index.checked_add(i).and_then(|idx| self.get(idx))
    }

    /// Advance while `pred` returns true, returning a slice of all consumed elements.
    ///
    /// After this call, `current()` points to the first element that did not
    /// satisfy the predicate (or past the end).
    pub(crate) fn consume_while(&mut self, pred: impl Fn(&T) -> bool) -> &'a [T] {
        let start = self.index;
        while self.index < self.values.len() && pred(&self.values[self.index]) {
            self.index += 1;
        }
        &self.values[start..self.index]
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.values.len() {
            return None;
        }
        Some(&self.values[index])
    }
}
