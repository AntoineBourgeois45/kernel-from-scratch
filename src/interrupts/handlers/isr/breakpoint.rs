use core::arch::asm;

use crate::{kprint, vga::terminal::LogLevel};

pub fn handle_breakpoint() {
    kprint!(LogLevel::Error, "Breakpoint exception\n");
    unsafe {
        asm!(
            "cli",
            "hlt",
            "ret",
            options(nostack)
        );
    }
}