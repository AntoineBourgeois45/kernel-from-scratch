use core::arch::asm;

use crate::kprint;
use crate::vga::terminal::LogLevel;

pub fn fatal(msg: &str) -> ! {
    kprint!(LogLevel::Error, "kernel panic: {}", msg);
    halt();
}

pub fn warn(msg: &str) {
    kprint!(LogLevel::Warning, "kernel warning: {}", msg);
}

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
