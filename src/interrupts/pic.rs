use core::arch::asm;

use crate::{interrupts::io::outb, ps2::mouse, vga::terminal::{LogLevel, Terminal}};

pub const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW1_ICW4: u8 = 0x01;
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
}

pub unsafe fn handle_interrupt(interrupt_number: u8) {
    match interrupt_number {
        0x2C => {
            mouse::handle_irq12();
        }
        _ => {
            if interrupt_number >= 0x28 && interrupt_number < 0x30 {
                outb(PIC2_CMD, 0x20);
            }
            outb(PIC1_CMD, 0x20);
        }
    }

    outb(PIC1_CMD, 0x20);
}