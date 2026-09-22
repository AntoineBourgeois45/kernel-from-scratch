use core::arch::asm;

const INTERRUPT_FLAG: u32 = 1 << 9;

extern "C" {
    fn cpu_halt_clean() -> !;
}

#[inline]
pub fn read_eflags() -> u32 {
    let flags: u32;
    unsafe {
        asm!(
            "pushfd",
            "pop {flags:e}",
            flags = out(reg) flags,
            options(nomem, preserves_flags),
        );
    }
    flags
}

#[inline]
pub fn interrupts_enabled() -> bool {
    read_eflags() & INTERRUPT_FLAG != 0
}

#[inline]
pub fn disable_interrupts() {
    unsafe { asm!("cli", options(nomem, nostack)) }
}

#[inline]
pub fn enable_interrupts() {
    unsafe { asm!("sti", options(nomem, nostack)) }
}

#[inline]
pub fn wait_for_interrupt() {
    unsafe { asm!("hlt", options(nomem, nostack)) }
}

#[inline]
pub fn read_cr2() -> u32 {
    let value: u32;
    unsafe {
        asm!("mov {value:e}, cr2", value = out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

#[inline]
pub fn read_esp() -> u32 {
    let value: u32;
    unsafe {
        asm!("mov {value:e}, esp", value = out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Restores the interrupt state which was active when the guard was created.
pub struct InterruptGuard {
    restore_interrupts: bool,
}

impl InterruptGuard {
    pub fn new() -> Self {
        let restore_interrupts = interrupts_enabled();
        disable_interrupts();
        Self { restore_interrupts }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.restore_interrupts {
            enable_interrupts();
        }
    }
}

/// Stops the CPU after clearing all general-purpose registers except ESP.
pub fn halt_clean() -> ! {
    unsafe { cpu_halt_clean() }
}
