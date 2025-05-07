use core::arch::asm;

use crate::ps2::keyboard::keyboard_handler;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    pub isr_low: u16,
    pub kernel_cs: u16,
    pub reserved: u8,
    pub attributes: u8,
    pub isr_high: u16
}

#[repr(C, align(16))]
pub struct Idtr {
    pub limit: u16,
    pub base: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    isr_low: 0,
    kernel_cs: 0,
    reserved: 0,
    attributes: 0,
    isr_high: 0,
}; 256];

static mut IDTR: Idtr = Idtr {
    limit: 0,
    base: 0,
};

pub unsafe fn set_handler(index: usize, handler_function_addr: u32, selector: u16, attributes: u8) {
    IDT[index].isr_low = handler_function_addr as u16;
    IDT[index].kernel_cs = selector;
    IDT[index].reserved = 0;
    IDT[index].attributes = attributes;
    IDT[index].isr_high = (handler_function_addr >> 16) as u16;
}

pub unsafe fn init_idt() {
    for i in 0..256 {
        set_handler(i, 0, 0, 0);
    }

    set_handler(33, keyboard_handler as u32, 0x08, 0x8E);

    let idtr = Idtr {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: &raw const IDT as *const _ as u32,
    };

    asm!(
        "lidt [{}]",
        in(reg) &idtr,
        options(readonly, nostack)
    );
    asm!(
        "sti",
        options(nomem, nostack)
    );
}