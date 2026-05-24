use core::arch::asm;

use crate::inputs::io::outb;
use crate::kprint;
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
            _ => kprint!(LogLevel::Warning, "Unknown command: {}", cmd),
        }
    }

    fn print_help(&self) {
        kprint!(LogLevel::Info, "=== KFS Shell ===");
        kprint!(LogLevel::Info, "help   - show this help");
        kprint!(LogLevel::Info, "stack  - dump kernel stack");
        kprint!(LogLevel::Info, "clear  - clear screen");
        kprint!(LogLevel::Info, "reboot - reboot via PS/2 controller");
        kprint!(LogLevel::Info, "halt   - halt CPU");
    }
}

static mut SHELL: Shell = Shell::new();

pub fn init() {
    unsafe {
        SHELL.prompt();
    }
}

pub fn handle_char(ch: char) {
    unsafe {
        SHELL.handle_char(ch);
    }
}

fn reboot() -> ! {
    unsafe {
        outb(0x64, 0xFE);
    }
    halt()
}

fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
