use crate::{kprint, ps2::keyboard, vga::terminal::LogLevel};

pub fn handle_keyboard(scancode: u8) {
    keyboard::handle_keyboard_input(scancode);
}
