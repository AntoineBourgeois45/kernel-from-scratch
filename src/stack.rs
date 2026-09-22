use core::arch::asm;

use crate::kprint;
use crate::vga::terminal::LogLevel;

extern "C" {
    static stack_bottom: u8;
    static stack_top: u8;
}

pub fn stack_bounds() -> (u32, u32) {
    (
        core::ptr::addr_of!(stack_bottom) as u32,
        core::ptr::addr_of!(stack_top) as u32,
    )
}

pub unsafe fn copy_stack_words(esp: u32, destination: &mut [u32]) -> usize {
    let (bottom, top) = stack_bounds();
    if esp < bottom || esp >= top {
        return 0;
    }

    let available = ((top - esp) / 4) as usize;
    let count = destination.len().min(available);
    let source = esp as *const u32;
    for i in 0..count {
        destination[i] = core::ptr::read_volatile(source.add(i));
    }
    count
}

pub fn print_saved_stack(esp: u32, words: &[u32]) {
    kprint!(LogLevel::Info, "=== Saved Kernel Stack ===");
    for (index, value) in words.iter().enumerate() {
        kprint!(
            LogLevel::Info,
            "[{:<02}] 0x{:08x}: 0x{:08x}",
            index,
            esp.wrapping_add((index * 4) as u32),
            value
        );
    }
    kprint!(LogLevel::Info, "==========================");
}

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
        let value = unsafe { core::ptr::read_volatile(ptr.add(i)) };
        kprint!(
            LogLevel::Info,
            "[{:<02}] 0x{:08x}: 0x{:08x}",
            i,
            (esp as usize + i * 4) as u32,
            value
        );
        i += 1;
    }

    kprint!(LogLevel::Info, "=========================");
}
