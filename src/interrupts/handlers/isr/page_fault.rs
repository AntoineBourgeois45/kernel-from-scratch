use core::arch::asm;

use crate::{kprint, vga::terminal::LogLevel};

pub fn handle_page_fault() {
    kprint!(LogLevel::Error, "Page fault exception\n");
    unsafe {
        asm!(
            "cli",
            "hlt",
            "ret",
            options(nostack)
        );
    }
}