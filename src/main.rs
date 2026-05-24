#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod ps2;
pub mod inputs;
pub mod libc;
pub mod gdt;
pub mod stack;
pub mod shell;

use core::panic::PanicInfo;
use ps2::keyboard::KeyboardState;
use vga::terminal::LogLevel;

use crate::{inputs::handlers::get_input_handler, vga::terminal::terminal};

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
 #######   ##       Version 0.2.1
     ##   ##  ##
     ##   ######

");
    shell::init();

    loop {
        unsafe {
            get_input_handler().poll_and_handle_input(&mut KEYBOARD_STATE);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
