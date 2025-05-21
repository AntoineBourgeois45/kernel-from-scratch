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

fn _force_division_by_zero() {
    unsafe {
        asm!(
            "mov eax, 42",
            "mov ebx, 0",
            "div ebx",
            options(nostack, nomem, preserves_flags)
        );
    }
}

#[no_mangle]
fn _force_breakpoint() {
    unsafe { asm!("int3"); }
}

// fn test_keyboard_input() {
//     kprint!(LogLevel::Info, "Test du clavier. Tapez quelque chose (enter pour terminer) :\n");
    
//     let mut input_buffer = [0u8; 64];
    
//     let count = ps2::keyboard::read_line(&mut input_buffer, 64);
    
//     kprint!(LogLevel::Info, "Vous avez tapé ({} caractères): ", count - 1);
//     unsafe {
//         terminal().write(&input_buffer[0..count-1]);
//     }
//     kprint!(LogLevel::Default, "\n");
// }

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
    
    kprint!(LogLevel::Trace, "Initializing GDT...");
    init_gdt();

    kprint!(LogLevel::Trace, "Initializing IDT...");
    init_idt();

    kprint!(LogLevel::Trace, "Enabling interrupts...");
    unsafe { asm!("sti") }
    kprint!(LogLevel::Info, "Interrupts enabled successfully\n");

    unsafe {
        asm!("int $0x2");
    }

    // test_keyboard_input();

    // _force_breakpoint();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
