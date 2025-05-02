use core::arch::asm;

use crate::ps2::keyboard::keyboard_handler;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub zero: u32,
}

#[repr(C, align(16))]
pub struct Idtr {
    pub limit: u16,
    pub base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    type_attr: 0,
    offset_mid: 0,
    offset_high: 0,
    zero: 0,
}; 256];

pub unsafe fn init_idt() {
    let handler_addr = keyboard_handler as u64;
    IDT[32+1].offset_low = handler_addr as u16;
    IDT[32+1].selector = 0x08;
    IDT[32+1].ist = 0;
    IDT[32+1].type_attr = 0x8E;
    IDT[32+1].offset_mid = (handler_addr >> 16) as u16;
    IDT[32+1].offset_high = (handler_addr >> 32) as u32;

    let idtr = Idtr {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: &raw const IDT as *const _ as u64,
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