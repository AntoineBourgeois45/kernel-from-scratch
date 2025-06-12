pub struct LineEditor {
    buffer: [u8; 256],
    length: usize,
    cursor_pos: usize,
}

impl LineEditor {
    pub const fn new() -> Self {
        Self {
            buffer: [0; 256],
            length: 0,
            cursor_pos: 0,
        }
    }

    pub fn insert_char(&mut self, c: u8) {
        if self.length < self.buffer.len() - 1 {
            for i in (self.cursor_pos..self.length).rev() {
                self.buffer[i + 1] = self.buffer[i];
            }
            self.buffer[self.cursor_pos] = c;
            self.length += 1;
            self.cursor_pos += 1;
        }
    }
    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            for i in self.cursor_pos..self.length {
                self.buffer[i - 1] = self.buffer[i];
            }
            self.length -= 1;
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.length {
            self.cursor_pos += 1;
        }
    }
    pub fn move_cursor_start(&mut self) {
        self.cursor_pos = 0;
    }
    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.length;
    }

    pub fn get_line(&self) -> &[u8] {
        &self.buffer[..self.length]
    }
    pub fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn clear(&mut self) {
        self.buffer = [0; 256];
        self.length = 0;
        self.cursor_pos = 0;
    }
}