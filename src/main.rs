#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod ps2;
pub mod interrupts;

use core::panic::PanicInfo;
use vga::terminal::LogLevel;

use crate::vga::terminal::Terminal;

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
        let mut terminal = Terminal {
            row: 0,
            column: 0,
            color: 0,
            buffer: 0xb8000 as *mut u16,
        };
        
        interrupts::pic::handle_interrupt(&mut terminal, interrupt_num);
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let mut terminal = Terminal {
        row: 0,
        column: 0,
        color: 0,
        buffer: 0xb8000 as *mut u16,
    };

    unsafe {
        terminal.initialize();
        terminal.print("    ###    ####
   ####   ##  ##
  ## ##       ##
 ##  ##     ###
 #######   ##
     ##   ##  ##
     ##   ######\n\n", LogLevel::Default);

        terminal.print("Initializing mouse...\n", LogLevel::Trace);

        // if ps2::mouse::init_mouse() {
        //     terminal.write_str("Mouse initialized successfully.\n");
        // } else {
        //     terminal.write_str("Mouse initialization failed.\n");
        // }

        // interrupts::pic::init(&mut terminal);

        // terminal.write_str("Interrupts initialized.\n");
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
