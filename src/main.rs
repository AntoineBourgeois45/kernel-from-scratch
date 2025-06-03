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
use ps2::{controller::PS2Controller, keyboard::KeyboardState};
use vga::terminal::LogLevel;
use multiboot::{MultibootInfo, MultibootMmapEntry};

use crate::vga::terminal::terminal;

static mut KEYBOARD_STATE: KeyboardState = KeyboardState {
    shift_pressed: false,
    ctrl_pressed: false,
    alt_pressed: false,
};

static mut PS2_CONTROLLER: PS2Controller = PS2Controller {
    mouse_cycle: 0,
    mouse_packet: [0; 3],
};

#[no_mangle]
pub extern "C" fn kernel_main(info: *const MultibootInfo) -> ! {
    unsafe { terminal().initialize();
        // PS2_CONTROLLER.init_mouse();
    }

    kprint!(LogLevel::Default, 
"    ###    ####
   ####   ##  ##
  ## ##       ##    Rust Kernel from scratch
 ##  ##     ###
 #######   ##       Version 0.1.0
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
            if PS2_CONTROLLER.has_data() {
                let data = PS2_CONTROLLER.read_data();
                
                if PS2_CONTROLLER.is_mouse_data() {
                    if let Some(mouse_event) = PS2_CONTROLLER.process_mouse_data(data) {
                        // handle_mouse_event(mouse_event);
                        kprint!(LogLevel::Default, "Mouse Event: {:?}", mouse_event);
                    }
                } else {
                    if let Some(character) = KEYBOARD_STATE.process_scancode(data) {
                        // handle_keyboard_input(character);
                        kprint!(LogLevel::Default, "Key Pressed: {}", character);
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
