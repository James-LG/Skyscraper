pub struct VecPointer<T>
where T : Copy {
    values: Vec<T>,
    pub index: usize,
}

impl<T> VecPointer<T>
where T : Copy {
    pub fn new(values: Vec<T>) -> VecPointer<T> {
        VecPointer {
            values,
            index: 0,
        }
    }

    pub fn current(&self) -> Option<T> {
        self.get(self.index)
    }

    pub fn back(&mut self) -> Option<T> {
        self.back_add(1)
    }

    pub fn back_add(&mut self, i: usize) -> Option<T> {
        self.index -= i;
        self.current()
    }

    pub fn next(&mut self) -> Option<T> {
        self.next_add(1)
    }

    pub fn next_add(&mut self, i: usize) -> Option<T> {
        self.index += i;
        self.current()
    }

    pub fn peek(&self) -> Option<T> {
        self.peek_add(1)
    }

    pub fn peek_add(&self, i: usize) -> Option<T> {
        self.get(self.index + i)
    }

    fn get(&self, index: usize) -> Option<T> {
        if index >= self.values.len() {
            return None;
        }
        Some(self.values[index])
    }
}