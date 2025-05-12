#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod ps2;
pub mod interrupts;
pub mod gdt;

use core::{arch::asm, panic::PanicInfo};
use interrupts::{idt::init_idt, pic::init_pic};
use vga::terminal::LogLevel;

use crate::vga::terminal::terminal;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }
    dest
}

#[no_mangle]
pub extern "C" fn handle_interrupt_wrapper(interrupt_num: u8) {
    unsafe {
        interrupts::pic::handle_interrupt(interrupt_num);
    }
}

fn force_division_by_zero() {
    unsafe {
        core::arch::asm!(
            "mov eax, 42",
            "mov ebx, 0",
            "div ebx",
            options(nostack, nomem, preserves_flags)
        );
    }
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

    kprint!(LogLevel::Trace, "Initializing IDT...");

    unsafe {
        init_pic(0x20, 0x28);
        init_idt()
    }

    force_division_by_zero();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
