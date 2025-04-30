use core::arch::asm;

use crate::{ps2::{controller::{inb, outb}, mouse}, vga::terminal::Terminal};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

pub unsafe fn init(terminal: &mut Terminal) {
    init_pic();

    terminal.write_str("Interrupts initialization...\n");
    asm!("sti", options(nomem, nostack));
}

unsafe fn init_pic() {
    let mask1 = inb(PIC1_DATA);
    let mask2 = inb(PIC2_DATA);
    
    outb(PIC1_COMMAND, ICW1_INIT);
    outb(PIC2_COMMAND, ICW1_INIT);

    outb(PIC1_DATA, 0x20);
    outb(PIC2_DATA, 0x28);

    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);

    outb(PIC1_DATA, ICW4_8086);
    outb(PIC2_DATA, ICW4_8086);

    outb(PIC1_DATA, mask1 & !(1 << 2));
    outb(PIC2_DATA, mask2 & !(1 << 4));
}

pub unsafe fn handle_interrupt(terminal: &mut Terminal, interrupt_number: u8) {
    match interrupt_number {
        0x2C => {
            mouse::handle_irq12(terminal);
        }
        _ => {
            if interrupt_number >= 0x28 && interrupt_number < 0x30 {
                outb(PIC2_COMMAND, 0x20);
            }
            outb(PIC1_COMMAND, 0x20);
        }
    }

    outb(PIC1_COMMAND, 0x20);
}