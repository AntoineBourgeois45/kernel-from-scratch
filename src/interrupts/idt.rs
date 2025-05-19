use core::arch::asm;

use crate::interrupts::handlers::isr;
use crate::interrupts::pic::init_pic;
use crate::{kprint, vga::terminal::LogLevel};

// #[repr(C, packed)]
// #[derive(Copy, Clone)]
// pub struct IdtEntry {
//     pub isr_low: u16,
//     pub kernel_cs: u16,
//     pub reserved: u8,
//     pub attributes: u8,
//     pub isr_high: u16
// }

// #[repr(C, align(16))]
// struct AlignedIdt {
//     entries: [IdtEntry; 256],
// }

// #[repr(C, packed)]
// pub struct Idtr {
//     pub limit: u16,
//     pub base: u32,
// }

// static mut IDT: AlignedIdt = AlignedIdt {
//     entries: [IdtEntry {
//         isr_low: 0,
//         kernel_cs: 0,
//         reserved: 0,
//         attributes: 0,
//         isr_high: 0,
//     }; 256],
// };

// static mut IDTR: Idtr = Idtr {
//     limit: 0,
//     base: 0,
// };

// #[no_mangle]
// pub extern "x86-interrupt" fn exception_handler() -> ! {
//     unsafe { asm!("cli", "hlt", options(noreturn)) }
// }

// pub fn idt_set_descriptor(vector: u8, isr: u32, flags: u8) {
//     let descriptor = &mut unsafe { IDT.entries[vector as usize] };

//     descriptor.isr_low = (isr as u32 & 0xFFFF) as u16;
//     descriptor.kernel_cs = 0x10;

//     descriptor.attributes = flags;
//     descriptor.isr_high = ((isr as u32) >> 16) as u16;
//     descriptor.reserved = 0;
// }

// static mut VECTORS: [bool; 256] = [false; 256]; 

// extern "C" {
//     static ISR_STUB_TABLE: [u32; 32];
// }

// pub fn init_idt() {
//     unsafe {
//         IDTR.base = &IDT.entries[0] as *const _ as u32;
//         IDTR.limit = (core::mem::size_of::<IdtEntry>() * 256 - 1) as u16;

//         for vector in 0..32 {
//             let addr = ISR_STUB_TABLE[vector as usize];
//             kprint!(LogLevel::Info, "ISR[{}] = 0x{:x}\n", vector, addr);

//             idt_set_descriptor(vector, ISR_STUB_TABLE[vector as usize], 0x8E);
//             VECTORS[vector as usize] = true;
//         }

//         asm!("lidt [{0}]", in(reg) &IDTR as *const Idtr, options(nostack));
//         asm!("sti");
//         kprint!(LogLevel::Info, "IDT initialized successfully\n");
//     }
// }

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

#[no_mangle]
static mut IDT: [IdtEntry; 256] = [IdtEntry {
    isr_low: 0,
    kernel_cs: 0,
    reserved: 0,
    attributes: 0,
    isr_high: 0,
}; 256];

#[no_mangle]
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

extern "C" {
    fn isr0();   fn isr1();   fn isr2();   fn isr3();
    fn isr4();   fn isr5();   fn isr6();   fn isr7();
    fn isr8();   fn isr9();   fn isr10();  fn isr11();
    fn isr12();  fn isr13();  fn isr14();  fn isr15();
    fn isr16();  fn isr17();  fn isr18();  fn isr19();
    fn isr20();  fn isr21();  fn isr22();  fn isr23();
    fn isr24();  fn isr25();  fn isr26();  fn isr27();
    fn isr28();  fn isr29();  fn isr30();  fn isr31();
    fn isr32();  fn isr33();  fn isr34();  fn isr35();
    fn isr36();  fn isr37();  fn isr38();  fn isr39();
    fn isr40();  fn isr41();  fn isr42();  fn isr43();
    fn isr44();  fn isr45();  fn isr46();  fn isr47();
}

extern "C" {
    static mut interrupt_number: u8;
}

fn handle_divide_by_zero() {
    kprint!(LogLevel::Error, "Divide by zero exception\n");
}

fn handle_debug() {
    kprint!(LogLevel::Error, "Debug exception\n");
}

fn handle_unknown(int_no: u8) {
    kprint!(LogLevel::Error, "Unknown exception: {}\n", int_no);
}

#[no_mangle]
extern "C" fn exception_handler() {
    let int_no = unsafe { interrupt_number };
    kprint!(LogLevel::Error, "Exception #{} occurred\n", int_no);
    match int_no {
        0  => handle_divide_by_zero(),
        1  => handle_debug(),
        3  => isr::breakpoint::handle_breakpoint(),
        8  => isr::double_fault::handle_double_fault(),
        14 => isr::page_fault::handle_page_fault(),
        n  => handle_unknown(n),
    }
}

pub fn init_idt() {
    unsafe {
        let handlers: [u32; 48] = [
            isr0 as u32,  isr1 as u32,  isr2 as u32,  isr3 as u32,
            isr4 as u32,  isr5 as u32,  isr6 as u32,  isr7 as u32,
            isr8 as u32,  isr9 as u32,  isr10 as u32, isr11 as u32,
            isr12 as u32, isr13 as u32, isr14 as u32, isr15 as u32,
            isr16 as u32, isr17 as u32, isr18 as u32, isr19 as u32,
            isr20 as u32, isr21 as u32, isr22 as u32, isr23 as u32,
            isr24 as u32, isr25 as u32, isr26 as u32, isr27 as u32,
            isr28 as u32, isr29 as u32, isr30 as u32, isr31 as u32,
            isr32 as u32, isr33 as u32, isr34 as u32, isr35 as u32,
            isr36 as u32, isr37 as u32, isr38 as u32, isr39 as u32,
            isr40 as u32, isr41 as u32, isr42 as u32, isr43 as u32,
            isr44 as u32, isr45 as u32, isr46 as u32, isr47 as u32,
        ];
        for (i, &addr) in handlers.iter().enumerate() {
            set_handler(i, addr, 0x10, 0x8E);
        }

        init_pic(0x20, 0x28);

        IDTR.limit = (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16;
        IDTR.base = &IDT as *const _ as u32;

        asm!(
            "lidt [{}]",
            in(reg) &IDTR as *const Idtr,
            options(readonly, nostack)
        );
    }
    kprint!(LogLevel::Info, "IDT initialized successfully\n");
}
