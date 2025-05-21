use crate::{kprint, vga::terminal::LogLevel};

pub fn handle_general_protection_fault() {
    kprint!(LogLevel::Error, "General protection fault exception\n");
    loop {
        
    }
}