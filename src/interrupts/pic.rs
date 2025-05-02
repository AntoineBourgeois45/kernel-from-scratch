use core::arch::asm;

use crate::{interrupts::idt::init_idt, ps2::mouse, vga::terminal::{LogLevel, Terminal}};

pub const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", out("al") result, in("dx") port, options(nomem, nostack));
    result
}

pub unsafe fn init(terminal: &mut Terminal) {
    terminal.print("Initializing PIC...\n", LogLevel::Trace);
    init_pic(32, 40);
    terminal.print("Initializing IDT...\n", LogLevel::Trace);
    init_idt();
    
    
    asm!("sti", options(nomem, nostack));
}

unsafe fn init_pic(offset1: u8, offset2: u8) {
    outb(PIC1_CMD, ICW1_INIT | ICW1_ICW4);
    outb(PIC2_CMD, ICW1_INIT | ICW1_ICW4);

    outb(PIC1_DATA, offset1);
    outb(PIC2_DATA, offset2);

    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);

    outb(PIC1_DATA, ICW4_8086);
    outb(PIC2_DATA, ICW4_8086);

    outb(PIC1_DATA, 0x00);
    outb(PIC2_DATA, 0x00);
}

pub unsafe fn handle_interrupt(terminal: &mut Terminal, interrupt_number: u8) {
    match interrupt_number {
        0x2C => {
            mouse::handle_irq12(terminal);
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