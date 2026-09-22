use crate::inputs::io::{inb, outb};

pub const MASTER_OFFSET: u8 = 32;
pub const SLAVE_OFFSET: u8 = 40;

const MASTER_COMMAND: u16 = 0x20;
const MASTER_DATA: u16 = 0x21;
const SLAVE_COMMAND: u16 = 0xA0;
const SLAVE_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;
const PIC_EOI: u8 = 0x20;
const OCW3_READ_ISR: u8 = 0x0B;

#[inline]
unsafe fn io_wait() {
    outb(0x80, 0);
}

/// Moves IRQ0..15 away from the CPU exception vectors and initially masks all
/// devices. Callers explicitly unmask only the IRQs they are ready to handle.
pub unsafe fn initialize() {
    outb(MASTER_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(SLAVE_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();

    outb(MASTER_DATA, MASTER_OFFSET);
    io_wait();
    outb(SLAVE_DATA, SLAVE_OFFSET);
    io_wait();

    // Master IRQ2 is connected to the slave; the slave identity is 2.
    outb(MASTER_DATA, 1 << 2);
    io_wait();
    outb(SLAVE_DATA, 2);
    io_wait();

    outb(MASTER_DATA, ICW4_8086);
    io_wait();
    outb(SLAVE_DATA, ICW4_8086);
    io_wait();

    outb(MASTER_DATA, 0xFF);
    outb(SLAVE_DATA, 0xFF);
}

pub unsafe fn unmask(irq: u8) {
    if irq < 8 {
        let mask = inb(MASTER_DATA) & !(1 << irq);
        outb(MASTER_DATA, mask);
    } else if irq < 16 {
        let slave_irq = irq - 8;
        outb(SLAVE_DATA, inb(SLAVE_DATA) & !(1 << slave_irq));
        // A slave interrupt cannot reach the CPU while the cascade is masked.
        outb(MASTER_DATA, inb(MASTER_DATA) & !(1 << 2));
    }
}

pub unsafe fn mask(irq: u8) {
    if irq < 8 {
        outb(MASTER_DATA, inb(MASTER_DATA) | (1 << irq));
    } else if irq < 16 {
        let slave_irq = irq - 8;
        outb(SLAVE_DATA, inb(SLAVE_DATA) | (1 << slave_irq));
    }
}

unsafe fn read_isr(command_port: u16) -> u8 {
    outb(command_port, OCW3_READ_ISR);
    inb(command_port)
}

/// Returns true for the two spurious PIC interrupt cases. A spurious IRQ15
/// still needs an EOI on the master because its cascade line was asserted.
pub unsafe fn acknowledge_spurious(irq: u8) -> bool {
    if irq == 7 && read_isr(MASTER_COMMAND) & 0x80 == 0 {
        return true;
    }
    if irq == 15 && read_isr(SLAVE_COMMAND) & 0x80 == 0 {
        outb(MASTER_COMMAND, PIC_EOI);
        return true;
    }
    false
}

pub unsafe fn end_of_interrupt(irq: u8) {
    if irq >= 8 {
        outb(SLAVE_COMMAND, PIC_EOI);
    }
    outb(MASTER_COMMAND, PIC_EOI);
}
