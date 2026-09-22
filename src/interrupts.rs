use core::arch::asm;
use core::mem::size_of;
use core::ptr::write_volatile;

use crate::kprint;
use crate::ps2::keyboard::{keyboard_has_data, keyboard_read_scancode};
use crate::signals::KernelSignal;
use crate::vga::terminal::LogLevel;

const IDT_ENTRIES: usize = 256;
const STUB_COUNT: usize = 49;
const KERNEL_CODE_SELECTOR: u16 = 0x08;
const INTERRUPT_GATE: u8 = 0x8E;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    attributes: u8,
    offset_high: u16,
}

impl IdtEntry {
    const MISSING: Self = Self {
        offset_low: 0,
        selector: 0,
        zero: 0,
        attributes: 0,
        offset_high: 0,
    };

    fn interrupt_gate(handler: u32) -> Self {
        Self {
            offset_low: handler as u16,
            selector: KERNEL_CODE_SELECTOR,
            zero: 0,
            attributes: INTERRUPT_GATE,
            offset_high: (handler >> 16) as u16,
        }
    }
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: u32,
}

/// Exact stack layout built by `interrupt_common` in interrupts.asm.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct InterruptFrame {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub saved_esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,
    pub vector: u32,
    pub error_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
}

impl InterruptFrame {
    /// At ring 0 the interrupted ESP points immediately past EFLAGS.
    pub fn interrupted_esp(&self) -> u32 {
        self as *const Self as u32 + size_of::<Self>() as u32
    }
}

static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry::MISSING; IDT_ENTRIES];

extern "C" {
    static interrupt_stub_table: [u32; STUB_COUNT];
    fn isr_default();
}

unsafe fn install_and_load_idt() {
    let idt = core::ptr::addr_of_mut!(IDT) as *mut IdtEntry;
    let fallback = isr_default as *const () as usize as u32;

    for vector in 0..IDT_ENTRIES {
        write_volatile(idt.add(vector), IdtEntry::interrupt_gate(fallback));
    }
    for vector in 0..STUB_COUNT {
        write_volatile(
            idt.add(vector),
            IdtEntry::interrupt_gate(interrupt_stub_table[vector]),
        );
    }

    let idtr = Idtr {
        limit: (size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
        base: idt as u32,
    };
    asm!("lidt [{idtr:e}]", idtr = in(reg) &idtr, options(readonly, nostack, preserves_flags));
}

pub fn initialize() {
    crate::cpu::disable_interrupts();
    unsafe {
        install_and_load_idt();
        crate::pic::initialize();
        crate::pit::initialize(crate::pit::TIMER_HZ);
        crate::pic::unmask(0); // PIT timer
        crate::pic::unmask(1); // PS/2 keyboard
    }
    crate::cpu::enable_interrupts();
}

fn exception_name(vector: u32) -> &'static str {
    match vector {
        0 => "division by zero",
        1 => "debug",
        2 => "non-maskable interrupt",
        3 => "breakpoint",
        4 => "overflow",
        5 => "bound range exceeded",
        6 => "invalid opcode",
        7 => "device not available",
        8 => "double fault",
        9 => "coprocessor segment overrun",
        10 => "invalid TSS",
        11 => "segment not present",
        12 => "stack-segment fault",
        13 => "general protection fault",
        14 => "page fault",
        15 => "reserved",
        16 => "x87 floating-point exception",
        17 => "alignment check",
        18 => "machine check",
        19 => "SIMD floating-point exception",
        20 => "virtualization exception",
        21 => "control-protection exception",
        28 => "hypervisor injection exception",
        29 => "VMM communication exception",
        30 => "security exception",
        _ => "reserved exception",
    }
}

fn handle_exception(frame: &mut InterruptFrame) {
    match frame.vector {
        1 => {
            kprint!(LogLevel::Debug, "Debug exception at 0x{:08x}", frame.eip);
        }
        3 => {
            kprint!(LogLevel::Debug, "Breakpoint at 0x{:08x}", frame.eip);
            crate::signals::emit(KernelSignal::Breakpoint);
        }
        _ => crate::kpanic::fatal_exception(exception_name(frame.vector), frame),
    }
}

fn handle_irq(vector: u32) {
    let irq = (vector - crate::pic::MASTER_OFFSET as u32) as u8;
    unsafe {
        if crate::pic::acknowledge_spurious(irq) {
            return;
        }
    }

    match irq {
        0 => crate::signals::on_timer_interrupt(),
        1 => {
            if keyboard_has_data() {
                let scancode = keyboard_read_scancode();
                unsafe {
                    let keyboard_state = &mut *core::ptr::addr_of_mut!(crate::KEYBOARD_STATE);
                    crate::inputs::handlers::get_input_handler()
                        .handle_scancode(keyboard_state, scancode);
                }
                crate::signals::emit(KernelSignal::Keyboard);
            }
        }
        other => crate::signals::emit(KernelSignal::HardwareIrq(other)),
    }

    unsafe { crate::pic::end_of_interrupt(irq) };
}

#[no_mangle]
pub extern "C" fn rust_interrupt_dispatch(frame: &mut InterruptFrame) {
    match frame.vector {
        0..=31 => handle_exception(frame),
        32..=47 => handle_irq(frame.vector),
        48 => crate::signals::emit(KernelSignal::Software),
        vector => kprint!(LogLevel::Warning, "Unhandled interrupt vector {}", vector),
    }
}

pub fn trigger_breakpoint() {
    unsafe { asm!("int3", options(nomem, nostack)) }
}

pub fn trigger_invalid_opcode() {
    unsafe { asm!("ud2", options(nomem, nostack)) }
}

pub fn trigger_general_protection_fault() -> ! {
    unsafe { asm!("mov ax, 0xffff", "mov ds, ax", options(noreturn),) }
}
