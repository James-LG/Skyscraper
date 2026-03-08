/// A read-only pointer into a slice, supporting sequential traversal with peek and rewind.
///
/// Used internally by the HTML tokenizer and parser to iterate over character
/// streams while supporting lookahead and backtracking.
pub struct VecPointerRef<'a, T> {
    values: &'a [T],
    /// The current index position within the slice.
    pub(crate) index: usize,
}

impl<'a, T> VecPointerRef<'a, T> {
    /// Create a new pointer starting at position 0.
    pub fn new(values: &[T]) -> VecPointerRef<T> {
        VecPointerRef { values, index: 0 }
    }

    /// Returns `true` if the current index points to a valid element.
    pub fn has_next(&self) -> bool {
        self.get(self.index).is_some()
    }

    /// Returns a reference to the element at the current index, or `None` if past the end.
    pub fn current(&self) -> Option<&T> {
        self.get(self.index)
    }

    /// Returns the element at the current index and advances by one.
    pub fn next(&mut self) -> Option<&T> {
        self.next_add(1)
    }

    /// Returns the element at the current index and advances by `i` positions.
    pub fn next_add(&mut self, i: usize) -> Option<&T> {
        let index = self.index;
        self.index += i;
        self.get(index)
    }

    /// Moves the index back by one and returns the element at the new position.
    pub fn prev(&mut self) -> Option<&T> {
        self.prev_sub(1)
    }

    /// Moves the index back by `i` positions and returns the element at the new position.
    ///
    /// Returns `None` if `i` is greater than the current index (would underflow).
    pub fn prev_sub(&mut self, i: usize) -> Option<&T> {
        if i > self.index {
            return None;
        } else {
            self.index -= i;
        }
        self.current()
    }

    /// Returns a reference to the element one position ahead without advancing.
    pub fn peek(&self) -> Option<&T> {
        self.peek_add(1)
    }

    /// Returns a reference to the element `i` positions ahead without advancing.
    pub fn peek_add(&self, i: usize) -> Option<&T> {
        self.get(self.index + i)
    }

    /// Returns references to the next `num` elements without advancing.
    ///
    /// Stops early if the end of the slice is reached, so the returned
    /// vector may contain fewer than `num` elements.
    pub fn peek_multiple(&self, num: usize) -> Vec<&T> {
        let mut result = Vec::new();
        for i in 1..=num {
            if let Some(value) = self.peek_add(i) {
                result.push(value);
            } else {
                break;
            }
        }
        result
    }

    /// Returns references to the current element and the next `i - 1` elements without advancing.
    ///
    /// Stops early if the end of the slice is reached.
    pub fn peek_current_and_multiple(&self, i: usize) -> Vec<&T> {
        let mut result = Vec::new();
        for j in 0..i {
            if let Some(value) = self.peek_add(j) {
                result.push(value);
            } else {
                break;
            }
        }
        result
    }

    /// Advance while `pred` returns true, returning a slice of all consumed elements.
    ///
    /// After this call, `current()` points to the first element that did not
    /// satisfy the predicate (or past the end).
    pub fn consume_while(&mut self, pred: impl Fn(&T) -> bool) -> &'a [T] {
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
