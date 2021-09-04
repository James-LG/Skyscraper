pub struct TextPointer {
    text: Vec<char>,
    pub index: usize,
}

impl TextPointer {
    pub fn new(text: &str) -> TextPointer {
        let text: Vec<char> = text.chars().collect();
        TextPointer {
            text,
            index: 0,
        }
    }

    pub fn current(&self) -> Option<char> {
        self.get_char(self.index)
    }

    pub fn next_char(&mut self) -> Option<char> {
        self.next_char_add(1)
    }

    pub fn next_char_add(&mut self, i: usize) -> Option<char> {
        self.index += i;
        self.current()
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_add(1)
    }

    pub fn peek_add(&self, i: usize) -> Option<char> {
        self.get_char(self.index + i)
    }

    fn get_char(&self, index: usize) -> Option<char> {
        if index >= self.text.len() {
            return None;
        }
        Some(self.text[index])
    }
}