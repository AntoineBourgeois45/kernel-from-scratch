use core::ptr::{write_volatile, read_volatile};

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

#[inline]
pub const fn vga_entry_color(fg: VgaColor, bg: VgaColor) -> u8 {
    (bg as u8) << 4 | (fg as u8)
}

#[inline]
pub const fn vga_entry(c: u8, color: u8) -> u16 {
    c as u16 | (color as u16) << 8
}

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

pub struct Terminal {
    pub row: usize,
    pub column: usize,
    pub color: u8,
    pub buffer: *mut u16,
}

impl Terminal {
    pub unsafe fn initialize(&mut self) {
        self.row = 0;
        self.column = 0;
        self.color = vga_entry_color(VgaColor::LightGrey, VgaColor::Black);
        self.buffer = 0xb8000 as *mut u16;
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let index = y * VGA_WIDTH + x;
                write_volatile(self.buffer.add(index), vga_entry(b' ', self.color));
            }
        }
    }

    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }

    pub unsafe fn put_entry_at(&mut self, c: u8, color: u8, x: usize, y: usize) {
        let index = y * VGA_WIDTH + x;
        write_volatile(self.buffer.add(index), vga_entry(c, color));
    }

    pub unsafe fn scroll(&mut self) {
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let from = (y * VGA_WIDTH + x) as isize;
                let to = ((y - 1) * VGA_WIDTH + x) as isize;
                let val = read_volatile(self.buffer.add(from as usize));
                write_volatile(self.buffer.add(to as usize), val);
            }
        }
        let blank = vga_entry(b' ', self.color);
        let last_row = (VGA_HEIGHT - 1) * VGA_WIDTH;
        for x in 0..VGA_WIDTH {
            write_volatile(self.buffer.add(last_row + x), blank);
        }
        self.row -= VGA_HEIGHT - 1;
    }

    pub unsafe fn new_line(&mut self) {
        self.column = 0;
        self.row += 1;
        if self.row == VGA_HEIGHT {
            self.scroll();
        }
    }

    pub unsafe fn put_char(&mut self, c: u8) {
        match c {
            b'\n' => self.new_line(),
            byte => {
                self.put_entry_at(byte, self.color, self.column, self.row);
                self.column += 1;
                if self.column == VGA_WIDTH {
                    self.column = 0;
                    self.row += 1;
                    if self.row == VGA_HEIGHT {
                        self.row = 0;
                    }
                }
            }
        }
    }

    pub unsafe fn write(&mut self, data: &[u8]) {
        for &byte in data {
            self.put_char(byte);
        }
    }

    pub unsafe fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
    }
}