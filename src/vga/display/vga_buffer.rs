use core::ptr::write_volatile;

use crate::vga::terminal_manager::{VGA_BUFFER_SIZE, VGA_HEIGHT, VGA_WIDTH};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Default,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub const fn prefix(self) -> Option<&'static str> {
        match self {
            LogLevel::Error => Some("error: "),
            LogLevel::Warning => Some("warning: "),
            LogLevel::Info => Some("info: "),
            LogLevel::Debug => Some("debug: "),
            LogLevel::Trace => Some("trace: "),
            LogLevel::Default => None,
        }
    }
    pub const fn color(self) -> u8 {
        match self {
            LogLevel::Error => vga_entry_color(VgaColor::LightRed, VgaColor::Black),
            LogLevel::Warning => vga_entry_color(VgaColor::LightBrown, VgaColor::Black),
            LogLevel::Info => vga_entry_color(VgaColor::LightGreen, VgaColor::Black),
            LogLevel::Debug => vga_entry_color(VgaColor::LightBlue, VgaColor::Black),
            LogLevel::Trace => vga_entry_color(VgaColor::LightMagenta, VgaColor::Black),
            LogLevel::Default => vga_entry_color(VgaColor::White, VgaColor::Black),
        }
    }
}

#[inline]
pub const fn vga_entry_color(fg: VgaColor, bg: VgaColor) -> u8 {
    (bg as u8) << 4 | (fg as u8)
}

#[inline]
pub const fn vga_entry(c: u8, color: u8) -> u16 {
    c as u16 | (color as u16) << 8
}

pub struct VgaBuffer {
    pub color: u8,
    pub buffer: *mut u16,
}

impl VgaBuffer {
    pub const fn new() -> Self {
        Self {
            color: vga_entry_color(VgaColor::White, VgaColor::Black),
            buffer: 0xb8000 as *mut u16,
        }
    }

    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }
    
    pub unsafe fn refresh_screen(&mut self, screen_buffer: &[u16; VGA_BUFFER_SIZE]) {
        for (i, &entry) in screen_buffer.iter().enumerate() {
            write_volatile(self.buffer.add(i), entry);
        }
    }

    pub unsafe fn clear_vga_screen(&mut self) {
        let blank = vga_entry(b' ', self.color);
        for i in 0..VGA_BUFFER_SIZE {
            write_volatile(self.buffer.add(i), blank);
        }
    }

    pub unsafe fn scroll_up(&mut self, screen_buffer: &mut [u16; VGA_BUFFER_SIZE]) {
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let from = y * VGA_WIDTH + x;
                let to = (y - 1) * VGA_WIDTH + x;
                screen_buffer[to] = screen_buffer[from];
            }
        }

        let blank = vga_entry(b' ', self.color);
        let last_row = (VGA_HEIGHT - 1) * VGA_WIDTH;
        for x in 0..VGA_WIDTH {
            screen_buffer[last_row + x] = blank;
        }

        self.refresh_screen(screen_buffer);
    }

    pub unsafe fn scroll_down(&mut self, screen_buffer: &mut [u16; VGA_BUFFER_SIZE]) {
        for y in (0..VGA_HEIGHT - 1).rev() {
            for x in 0..VGA_WIDTH {
                let from = y * VGA_WIDTH + x;
                let to = (y + 1) * VGA_WIDTH + x;
                screen_buffer[to] = screen_buffer[from];
            }
        }

        let blank = vga_entry(b' ', self.color);
        let last_row = (VGA_HEIGHT - 1) * VGA_WIDTH;
        for x in 0..VGA_WIDTH {
            screen_buffer[last_row + x] = blank;
        }

        self.refresh_screen(screen_buffer);
    }

}