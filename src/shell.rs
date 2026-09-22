use crate::inputs::io::outb;
use crate::kprint;
use crate::signals::{self, KernelSignal};
use crate::stack;
use crate::vga::terminal::{terminal, LogLevel};

const BUFFER_SIZE: usize = 128;

pub struct Shell {
    buf: [u8; BUFFER_SIZE],
    len: usize,
}

impl Shell {
    pub const fn new() -> Self {
        Self {
            buf: [0; BUFFER_SIZE],
            len: 0,
        }
    }

    pub fn prompt(&self) {
        unsafe { terminal().write_str("> ") };
    }

    pub fn handle_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                unsafe { terminal().put_char(b'\n') };
                self.execute_current();
                self.len = 0;
                self.prompt();
            }
            '\x08' => {
                if self.len > 0 {
                    self.len -= 1;
                    unsafe { terminal().put_char(b'\x08') };
                }
            }
            _ => {
                if ch.is_ascii() && !ch.is_ascii_control() && self.len < BUFFER_SIZE - 1 {
                    self.buf[self.len] = ch as u8;
                    self.len += 1;
                    unsafe { terminal().put_char(ch as u8) };
                }
            }
        }
    }

    fn execute_current(&mut self) {
        let cmd = match core::str::from_utf8(&self.buf[..self.len]) {
            Ok(s) => s.trim(),
            Err(_) => {
                kprint!(LogLevel::Error, "Invalid command encoding");
                return;
            }
        };

        if cmd.is_empty() {
            return;
        }

        match cmd {
            "help" => self.print_help(),
            "stack" => stack::dump_stack(32),
            "clear" => unsafe { terminal().clear_screen() },
            "halt" => halt(),
            "reboot" => reboot(),
            "int3" => crate::interrupts::trigger_breakpoint(),
            "softint" => signals::trigger_software_interrupt(),
            "signal" => self.schedule_test_signal(),
            "ticks" => kprint!(LogLevel::Info, "timer ticks: {}", signals::ticks()),
            "panic" => panic!("panic requested from the kernel shell"),
            "fault" => crate::interrupts::trigger_invalid_opcode(),
            "gpf" => crate::interrupts::trigger_general_protection_fault(),
            _ => kprint!(LogLevel::Warning, "Unknown command: {}", cmd),
        }
    }

    fn schedule_test_signal(&self) {
        match signals::schedule_after(KernelSignal::Software, crate::pit::TIMER_HZ as u64) {
            Ok(()) => kprint!(LogLevel::Info, "software signal scheduled in one second"),
            Err(error) => kprint!(LogLevel::Error, "cannot schedule signal: {:?}", error),
        }
    }

    fn print_help(&self) {
        kprint!(LogLevel::Info, "=== KFS Shell ===");
        kprint!(LogLevel::Info, "help   - show this help");
        kprint!(LogLevel::Info, "stack  - dump kernel stack");
        kprint!(LogLevel::Info, "clear  - clear screen");
        kprint!(LogLevel::Info, "reboot - reboot via PS/2 controller");
        kprint!(LogLevel::Info, "halt   - halt CPU");
        kprint!(LogLevel::Info, "int3   - test the breakpoint exception");
        kprint!(LogLevel::Info, "softint- trigger the kernel software interrupt");
        kprint!(LogLevel::Info, "signal - schedule a software signal in one second");
        kprint!(LogLevel::Info, "ticks  - show the timer tick counter");
        kprint!(LogLevel::Info, "panic  - test the global panic handler");
        kprint!(LogLevel::Info, "fault  - trigger a fatal invalid-opcode exception");
        kprint!(LogLevel::Info, "gpf    - trigger a general-protection fault");
    }
}

static mut SHELL: Shell = Shell::new();

pub fn init() {
    let _ = signals::register_callback(KernelSignal::Software, software_signal_callback);
    unsafe {
        (&*core::ptr::addr_of!(SHELL)).prompt();
    }
}

fn software_signal_callback(_signal: KernelSignal) {
    kprint!(LogLevel::Info, "software signal callback executed");
}

pub fn handle_char(ch: char) {
    unsafe {
        (&mut *core::ptr::addr_of_mut!(SHELL)).handle_char(ch);
    }
}

fn reboot() -> ! {
    unsafe {
        outb(0x64, 0xFE);
    }
    halt()
}

fn halt() -> ! {
    crate::cpu::halt_clean()
}
