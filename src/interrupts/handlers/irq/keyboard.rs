use crate::{kprint, vga::terminal::LogLevel};

pub fn handle_keyboard(scancode: u8) {
    kprint!(LogLevel::Debug, "Keyboard scancode: {}\n", scancode);
}

