#![no_std]
#![no_main]
#![allow(unused_unsafe)]

pub mod vga;
pub mod ps2;
pub mod inputs;
pub mod libc;
pub mod gdt;
pub mod stack;
pub mod shell;
pub mod cpu;
pub mod pic;
pub mod pit;
pub mod signals;
pub mod interrupts;
pub mod kpanic;

use core::panic::PanicInfo;
use ps2::keyboard::KeyboardState;
use vga::terminal::LogLevel;

use crate::vga::terminal::terminal;

pub static mut KEYBOARD_STATE: KeyboardState = KeyboardState {
    shift_pressed: false,
    ctrl_pressed: false,
    alt_pressed: false,
    extended_mode: false,
};

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        gdt::init();
        terminal().initialize();
    }

    kprint!(LogLevel::Default, 
"    ###    ####
   ####   ##  ##
  ## ##       ##    Rust Kernel from scratch
 ##  ##     ###
 #######   ##       Version 0.4.0
     ##   ##  ##
     ##   ######

");
    shell::init();
    interrupts::initialize();
    kprint!(LogLevel::Info, "IDT loaded; timer and keyboard interrupts enabled");

    loop {
        cpu::wait_for_interrupt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kpanic::rust_panic(info)
}
