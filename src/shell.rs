use core::arch::asm;

use crate::inputs::io::outb;
use crate::kprint;
use crate::stack;
use crate::memory;
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
        kprint!(LogLevel::Default, "> ");
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

        let (argc, args) = Self::split_args(cmd);
        let verb = args[0];
        match verb {
            "help" => self.print_help(),
            "stack" => stack::dump_stack(32),
            "memtest" => self.run_memtest(),
            "meminfo" => unsafe { memory::print_info() },
            "memdemo" => self.run_memdemo(),
            "memdump" => self.run_memdump(argc, &args),
            "map" => self.run_map(argc, &args),
            "pmem" => self.run_pmem(argc, &args),
            "clear" => unsafe { terminal().clear_screen() },
            "halt" => halt(),
            "reboot" => reboot(),
            _ => kprint!(LogLevel::Warning, "Unknown command: {}", verb),
        }
    }

    fn print_help(&self) {
        kprint!(LogLevel::Info, "=== KFS Shell ===");
        kprint!(LogLevel::Info, "help   - show this help");
        kprint!(LogLevel::Info, "stack  - dump kernel stack");
        kprint!(LogLevel::Info, "memtest - test paging + allocators");
        kprint!(LogLevel::Info, "meminfo - show memory stats");
        kprint!(LogLevel::Info, "memdemo - alloc/free demo");
        kprint!(LogLevel::Info, "memdump <addr> <len> - dump bytes (hex)");
        kprint!(LogLevel::Info, "map <addr> - show page mapping");
        kprint!(LogLevel::Info, "pmem [start] [count] - list frames");
        kprint!(LogLevel::Info, "clear  - clear screen");
        kprint!(LogLevel::Info, "reboot - reboot via PS/2 controller");
        kprint!(LogLevel::Info, "halt   - halt CPU");
    }

    fn run_memtest(&self) {
        let ok = unsafe { memory::memtest() };
        if ok {
            kprint!(LogLevel::Info, "memtest: ok");
        } else {
            kprint!(LogLevel::Error, "memtest: failed");
        }
    }

    fn run_memdemo(&self) {
        kprint!(LogLevel::Info, "memdemo: before");
        unsafe { memory::print_info() };
        let a = unsafe { memory::kmalloc(256) };
        let b = unsafe { memory::vmalloc(memory::paging::PAGE_SIZE) };
        kprint!(LogLevel::Info, "kmalloc(256) -> 0x{:08x}", a as u32);
        kprint!(LogLevel::Info, "vmalloc(4KB) -> 0x{:08x}", b as u32);
        kprint!(LogLevel::Info, "memdemo: after alloc");
        unsafe { memory::print_info() };
        unsafe {
            memory::kfree(a);
            memory::vfree(b);
        }
        kprint!(LogLevel::Info, "memdemo: after free");
        unsafe { memory::print_info() };
    }

    fn run_memdump(&self, argc: usize, args: &[&str; 3]) {
        if argc < 2 {
            kprint!(LogLevel::Warning, "usage: memdump <addr> <len>");
            return;
        }
        let addr = match Self::parse_u32(args[1]) {
            Some(v) => v,
            None => {
                kprint!(LogLevel::Warning, "invalid addr");
                return;
            }
        };
        let len = if argc >= 3 {
            match Self::parse_usize(args[2]) {
                Some(v) => v,
                None => {
                    kprint!(LogLevel::Warning, "invalid len");
                    return;
                }
            }
        } else {
            64
        };
        unsafe { memory::memdump(addr, len) };
    }

    fn run_map(&self, argc: usize, args: &[&str; 3]) {
        if argc < 2 {
            kprint!(LogLevel::Warning, "usage: map <addr>");
            return;
        }
        let addr = match Self::parse_u32(args[1]) {
            Some(v) => v,
            None => {
                kprint!(LogLevel::Warning, "invalid addr");
                return;
            }
        };
        unsafe { memory::print_map(addr) };
    }

    fn run_pmem(&self, argc: usize, args: &[&str; 3]) {
        let start = if argc >= 2 {
            match Self::parse_usize(args[1]) {
                Some(v) => v,
                None => {
                    kprint!(LogLevel::Warning, "invalid start");
                    return;
                }
            }
        } else {
            0
        };
        let count = if argc >= 3 {
            match Self::parse_usize(args[2]) {
                Some(v) => v,
                None => {
                    kprint!(LogLevel::Warning, "invalid count");
                    return;
                }
            }
        } else {
            32
        };
        unsafe { memory::print_pmm_range(start, count) };
    }

    fn split_args(cmd: &str) -> (usize, [&str; 3]) {
        let mut parts = ["", "", ""];
        let bytes = cmd.as_bytes();
        let mut i = 0usize;
        let mut count = 0usize;
        while i < bytes.len() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if count < parts.len() {
                parts[count] = &cmd[start..i];
                count += 1;
            } else {
                break;
            }
        }
        (count, parts)
    }

    fn parse_u32(text: &str) -> Option<u32> {
        let s = text.as_bytes();
        if s.len() >= 2 && s[0] == b'0' && (s[1] == b'x' || s[1] == b'X') {
            return Self::parse_hex(&s[2..]);
        }
        Self::parse_dec(s)
    }

    fn parse_usize(text: &str) -> Option<usize> {
        Self::parse_u32(text).map(|v| v as usize)
    }

    fn parse_hex(bytes: &[u8]) -> Option<u32> {
        if bytes.is_empty() {
            return None;
        }
        let mut value: u32 = 0;
        for &b in bytes {
            let digit = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'a'..=b'f' => (b - b'a' + 10) as u32,
                b'A'..=b'F' => (b - b'A' + 10) as u32,
                _ => return None,
            };
            value = value.checked_mul(16)?;
            value = value.checked_add(digit)?;
        }
        Some(value)
    }

    fn parse_dec(bytes: &[u8]) -> Option<u32> {
        if bytes.is_empty() {
            return None;
        }
        let mut value: u32 = 0;
        for &b in bytes {
            if !(b'0'..=b'9').contains(&b) {
                return None;
            }
            value = value.checked_mul(10)?;
            value = value.checked_add((b - b'0') as u32)?;
        }
        Some(value)
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
