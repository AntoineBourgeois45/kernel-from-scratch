#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod ps2;
pub mod interrupts;
pub mod gdt;
pub mod libc;
pub mod multiboot;

use core::panic::PanicInfo;
use ps2::keyboard::KeyboardState;
use vga::terminal::LogLevel;
use multiboot::MultibootInfo;

use crate::{ps2::input_handler::get_input_handler, vga::terminal::terminal};

pub static mut KEYBOARD_STATE: KeyboardState = KeyboardState {
    shift_pressed: false,
    ctrl_pressed: false,
    alt_pressed: false,
    extended_mode: false,
};


#[no_mangle]
pub extern "C" fn kernel_main(info: *const MultibootInfo) -> ! {
    unsafe {
        terminal().initialize();
    }

    kprint!(LogLevel::Default, 
"    ###    ####
   ####   ##  ##
  ## ##       ##    Rust Kernel from scratch
 ##  ##     ###
 #######   ##       Version 0.2.0
     ##   ##  ##
     ##   ######

");

    // unsafe {
    //     let mmap_length = (*info).mmap_length;
    //     kprint!(LogLevel::Default, "{}", mmap_length);
    //     for i in 0.. mmap_length {
    //         let p = ((*info).mmap_addr + core::mem::size_of::<MultibootMmapEntry>() as u32 * i) as *const MultibootMmapEntry;
    //         let size = (*p).size;
    //         let len = (*p).len;
    //         let addr = (*p).addr;
    //         kprint!(LogLevel::Default, "size: {}, len: {}, addr: {}", size, len, addr);
    //     }
    // }

    loop {
        unsafe {
            get_input_handler().poll_and_handle_input(&mut KEYBOARD_STATE);
            small_delay();
        }
    }
}

unsafe fn small_delay() {
    for _ in 0..1000 {
        core::arch::asm!("nop");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
