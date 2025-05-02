use crate::interrupts::pic::{inb, outb, PIC1_CMD};

#[no_mangle]
pub extern "x86-interrupt" fn keyboard_handler(
    _interrupt_number: u8,
    _stack_frame: &mut (),
) {
    let scancode = unsafe { inb(0x60) };

    unsafe { outb(PIC1_CMD, 0x20); }
}