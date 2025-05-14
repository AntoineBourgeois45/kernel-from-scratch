#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod ps2;
pub mod interrupts;
pub mod gdt;

use core::{arch::asm, panic::PanicInfo};
use gdt::gdt::init_gdt;
use interrupts::idt::init_idt;
use vga::terminal::LogLevel;

use crate::vga::terminal::terminal;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }
    dest
}

fn force_division_by_zero() {
    unsafe {
        asm!(
            "mov eax, 42",
            "mov ebx, 0",
            "div ebx",
            options(nostack, nomem, preserves_flags)
        );
    }
}

fn force_breakpoint() {
    unsafe { asm!("int3"); }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe { terminal().initialize() }
    kprint!(LogLevel::Default, 
"    ###    ####
   ####   ##  ##
  ## ##       ##
 ##  ##     ###
 #######   ##
     ##   ##  ##
     ##   ######

");
    kprint!(LogLevel::Trace, "Disabling interrupts...\n");
    unsafe { asm!("cli", options(nomem, nostack)) }

    kprint!(LogLevel::Trace, "Initializing GDT...");
    init_gdt();

    kprint!(LogLevel::Trace, "Initializing IDT...");
    init_idt();

    kprint!(LogLevel::Trace, "Enabling interrupts...");
    unsafe { asm!("sti") }
    kprint!(LogLevel::Info, "Interrupts enabled successfully\n");

    // force_breakpoint();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
