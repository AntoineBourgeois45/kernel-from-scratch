use core::arch::asm;

use crate::kprint;
use crate::vga::terminal::LogLevel;

pub fn dump_stack(words: usize) {
    let esp: u32;
    unsafe {
        asm!("mov {0}, esp", out(reg) esp);
    }

    kprint!(LogLevel::Info, "=== Kernel Stack Dump ===");
    kprint!(LogLevel::Info, "esp=0x{:08x}", esp);

    let ptr = esp as *const u32;
    let mut i = 0usize;
    while i < words {
        unsafe {
            let value = core::ptr::read_volatile(ptr.add(i));
            kprint!(
                LogLevel::Info,
                "[{:<02}] 0x{:08x}: 0x{:08x}",
                i,
                (esp as usize + i * 4) as u32,
                value
            );
        }
        i += 1;
    }

    kprint!(LogLevel::Info, "=========================");
}
