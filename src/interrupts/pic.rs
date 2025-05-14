use crate::{interrupts::{handlers::IRQ, io::outb}, kprint, vga::terminal::LogLevel};

use super::io::inb;

pub const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

pub unsafe fn init_pic(offset1: u8, offset2: u8) {
    outb(PIC1_CMD, ICW1_INIT);
    outb(PIC2_CMD, ICW1_INIT);

    outb(PIC1_DATA, offset1);
    outb(PIC2_DATA, offset2);

    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);

    outb(PIC1_DATA, ICW4_8086);
    outb(PIC2_DATA, ICW4_8086);

    outb(PIC1_DATA, 0x00);
    outb(PIC2_DATA, 0x00);

    // for testing purposes
    outb(PIC1_DATA, 0xFD);
    outb(PIC2_DATA, 0xFF);
}

pub unsafe fn send_eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC2_CMD, 0x20);
    }
    outb(PIC1_CMD, 0x20);
}

#[no_mangle]
extern "C" fn handle_interrupt(interrupt_number: u8) {
    unsafe {
        match interrupt_number {
            32 => {
                IRQ::timer::handle_timer();
            }
            33 => {
                IRQ::keyboard::handle_keyboard(inb(0x60));
            }
            _ => {
                kprint!(LogLevel::Error, "Unhandled interrupt: {}\n", interrupt_number);
            }
        }
        send_eoi(interrupt_number);
    }
}
