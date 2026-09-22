use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::interrupts::InterruptFrame;
use crate::kprint;
use crate::vga::terminal::LogLevel;

const SAVED_STACK_WORDS: usize = 16;

#[derive(Clone, Copy)]
pub struct PanicSnapshot {
    pub valid: bool,
    pub vector: u32,
    pub error_code: u32,
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub stack: [u32; SAVED_STACK_WORDS],
    pub stack_len: usize,
}

impl PanicSnapshot {
    const EMPTY: Self = Self {
        valid: false,
        vector: 0,
        error_code: 0,
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
        esi: 0,
        edi: 0,
        ebp: 0,
        esp: 0,
        eip: 0,
        cs: 0,
        eflags: 0,
        stack: [0; SAVED_STACK_WORDS],
        stack_len: 0,
    };
}

static PANICKING: AtomicBool = AtomicBool::new(false);
static mut LAST_PANIC: PanicSnapshot = PanicSnapshot::EMPTY;

unsafe fn copy_stack(snapshot: &mut PanicSnapshot) {
    snapshot.stack_len = crate::stack::copy_stack_words(snapshot.esp, &mut snapshot.stack);
}

unsafe fn save_exception(frame: &InterruptFrame) {
    let mut snapshot = PanicSnapshot {
        valid: true,
        vector: frame.vector,
        error_code: frame.error_code,
        eax: frame.eax,
        ebx: frame.ebx,
        ecx: frame.ecx,
        edx: frame.edx,
        esi: frame.esi,
        edi: frame.edi,
        ebp: frame.ebp,
        esp: frame.interrupted_esp(),
        eip: frame.eip,
        cs: frame.cs,
        eflags: frame.eflags,
        stack: [0; SAVED_STACK_WORDS],
        stack_len: 0,
    };
    copy_stack(&mut snapshot);
    LAST_PANIC = snapshot;
}

unsafe fn save_rust_panic() {
    let mut snapshot = PanicSnapshot::EMPTY;
    snapshot.valid = true;
    snapshot.vector = u32::MAX;
    snapshot.esp = crate::cpu::read_esp();
    snapshot.eflags = crate::cpu::read_eflags();
    copy_stack(&mut snapshot);
    LAST_PANIC = snapshot;
}

fn begin_panic() {
    crate::cpu::disable_interrupts();
    if PANICKING.swap(true, Ordering::SeqCst) {
        crate::cpu::halt_clean();
    }
}

fn print_snapshot(snapshot: PanicSnapshot) {
    kprint!(
        LogLevel::Error,
        "vector={} error=0x{:08x} eip=0x{:08x} cs=0x{:04x} eflags=0x{:08x}",
        snapshot.vector,
        snapshot.error_code,
        snapshot.eip,
        snapshot.cs,
        snapshot.eflags
    );
    kprint!(
        LogLevel::Error,
        "eax={:08x} ebx={:08x} ecx={:08x} edx={:08x}",
        snapshot.eax,
        snapshot.ebx,
        snapshot.ecx,
        snapshot.edx
    );
    kprint!(
        LogLevel::Error,
        "esi={:08x} edi={:08x} ebp={:08x} esp={:08x}",
        snapshot.esi,
        snapshot.edi,
        snapshot.ebp,
        snapshot.esp
    );
    crate::stack::print_saved_stack(snapshot.esp, &snapshot.stack[..snapshot.stack_len]);
}

pub fn fatal_exception(name: &str, frame: &InterruptFrame) -> ! {
    begin_panic();
    unsafe { save_exception(frame) };

    kprint!(LogLevel::Error, "=== FATAL CPU EXCEPTION ===");
    kprint!(LogLevel::Error, "{}", name);
    if frame.vector == 14 {
        kprint!(
            LogLevel::Error,
            "fault address (CR2)=0x{:08x}",
            crate::cpu::read_cr2()
        );
    }
    let snapshot = unsafe { LAST_PANIC };
    print_snapshot(snapshot);
    crate::cpu::halt_clean()
}

pub fn rust_panic(info: &PanicInfo) -> ! {
    begin_panic();
    unsafe { save_rust_panic() };

    kprint!(LogLevel::Error, "=== KERNEL PANIC ===");
    kprint!(LogLevel::Error, "{}", info);
    let snapshot = unsafe { LAST_PANIC };
    print_snapshot(snapshot);
    crate::cpu::halt_clean()
}

pub fn last_snapshot() -> PanicSnapshot {
    let _guard = crate::cpu::InterruptGuard::new();
    unsafe { LAST_PANIC }
}
