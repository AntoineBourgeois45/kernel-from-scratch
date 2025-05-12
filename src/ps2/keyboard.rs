use crate::{interrupts::{io::{inb, outb}, pic::PIC1_CMD}, kprint};

#[no_mangle]
pub extern "x86-interrupt" fn keyboard_handler(
    _interrupt_number: u8,
    _stack_frame: &mut (),
) {
    let scancode = unsafe { inb(0x60) };

    kprint!(
        crate::vga::terminal::LogLevel::Default,
        "Scancode: {:#x}",
        scancode
    );

    unsafe { outb(PIC1_CMD, 0x20); }
}