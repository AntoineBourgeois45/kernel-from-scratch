use core::arch::asm;

use crate::{kprint, vga::terminal::LogLevel};

pub fn handle_double_fault() {
    kprint!(LogLevel::Error, "Double fault exception\n");
    loop {
        
    }
}