pub struct VecPointerRef<'a, T> {
    values: &'a [T],
    pub index: usize,
}

impl<'a, T> VecPointerRef<'a, T> {
    pub fn new(values: &[T]) -> VecPointerRef<T> {
        VecPointerRef { values, index: 0 }
    }

    pub fn has_next(&self) -> bool {
        self.get(self.index).is_some()
    }

    pub fn current(&self) -> Option<&T> {
        self.get(self.index)
    }

    pub fn next(&mut self) -> Option<&T> {
        self.next_add(1)
    }

    pub fn next_add(&mut self, i: usize) -> Option<&T> {
        self.index += i;
        self.current()
    }

    pub fn prev(&mut self) -> Option<&T> {
        self.prev_sub(1)
    }

    pub fn prev_sub(&mut self, i: usize) -> Option<&T> {
        if i > self.index {
            return None;
        } else {
            self.index -= i;
        }
        self.current()
    }

    pub fn peek(&self) -> Option<&T> {
        self.peek_add(1)
    }

    pub fn peek_add(&self, i: usize) -> Option<&T> {
        self.get(self.index + i)
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.values.len() {
            return None;
        }
        Some(&self.values[index])
    }
}
