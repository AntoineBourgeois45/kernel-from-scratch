#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod ps2;
pub mod inputs;
pub mod libc;

use core::panic::PanicInfo;
use ps2::keyboard::KeyboardState;

use crate::{inputs::handlers::get_input_handler, vga::{display::{vga_buffer::LogLevel, idle::draw_idle_frame}, terminal_manager::terminal}};

pub static mut KEYBOARD_STATE: KeyboardState = KeyboardState {
    shift_pressed: false,
    ctrl_pressed: false,
    alt_pressed: false,
    extended_mode: false,
};

#[inline(always)]
unsafe fn rdtsc() -> u64 {
    let hi: u32;
    let lo: u32;
    core::arch::asm!(
        "rdtsc",
        out("edx") hi,
        out("eax") lo,
        options(nomem, nostack, preserves_flags)
    );
    ((hi as u64) << 32) | (lo as u64)
}

const CPU_FREQ_HZ:   u64 = 5_000_000_000;
const FRAME_RATE:    u64 = 24;
const CYCLES_PER_FRAME: u64 = CPU_FREQ_HZ / FRAME_RATE;


#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe { terminal().initialize(); }

    let mut frame      = 0;
    let mut last_tsc   = unsafe { rdtsc() };

    loop {
        if terminal().current_screen == 0 {
            let now = unsafe { rdtsc() };
            if now.wrapping_sub(last_tsc) >= CYCLES_PER_FRAME {
                unsafe { draw_idle_frame(frame) };
                frame = frame.wrapping_add(1);
                last_tsc = now;
            }
        }

        unsafe {
            get_input_handler().poll_and_handle_input(&mut KEYBOARD_STATE)
        };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
