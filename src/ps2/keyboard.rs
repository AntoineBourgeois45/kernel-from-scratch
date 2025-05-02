use crate::{interrupts::pic::{inb, outb, PIC1_CMD}, vga::terminal::{LogLevel, Terminal}};

#[no_mangle]
pub extern "x86-interrupt" fn keyboard_handler(
    _interrupt_number: u8,
    _stack_frame: &mut (),
) {
    let _scancode = unsafe { inb(0x60) };

    let mut terminal = Terminal {
        row: 0,
        column: 0,
        color: 0,
        buffer: 0xb8000 as *mut u16,
    };

    unsafe {
        terminal.print("a", LogLevel::Default);
    }

    unsafe { outb(PIC1_CMD, 0x20); }
}