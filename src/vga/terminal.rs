use core::{convert::TryInto, ptr::{read_volatile, write_volatile}};

use crate::inputs::io::{inb, outb};

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

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_SIZE: usize = VGA_WIDTH * VGA_HEIGHT;

const SCREENS_NUMBER: usize = 3;

const VGA_CRTC_ADDR: u16 = 0x3D4;
const VGA_CRTC_DATA: u16 = 0x3D5;
const VGA_CURSOR_LOC_HIGH: u8 = 0x0E;
const VGA_CURSOR_LOC_LOW: u8 = 0x0F;

pub struct Terminal {
    pub row: usize,
    pub column: usize,
    pub color: u8,
    pub buffer: *mut u16,
    pub cursor_visible: bool,

    pub current_screen: usize,
    pub screen_buffers: [[u16; VGA_BUFFER_SIZE]; SCREENS_NUMBER],
    pub screen_cursors: [(usize, usize); SCREENS_NUMBER],
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { self.write_str(s) };
        Ok(())
    }
}

impl Terminal {
    pub unsafe fn initialize(&mut self) {
        self.color = vga_entry_color(VgaColor::White, VgaColor::Black);
        self.buffer = 0xb8000 as *mut u16;
        self.cursor_visible = true;

        self.current_screen = 0;
        self.screen_cursors = [(0, 0); SCREENS_NUMBER];

        let blank = vga_entry(b' ', self.color);
        for screen in 0..SCREENS_NUMBER {
            for i in 0..VGA_BUFFER_SIZE {
                self.screen_buffers[screen][i] = blank;
            }
        }
        self.enable_cursor();
        self.refresh_screen();
        self.update_cursor();
    }

    pub fn switch_screen(&mut self, screen_id: usize) -> bool {
        if screen_id >= SCREENS_NUMBER {
            return false;
        }

        self.screen_cursors[self.current_screen] = (self.row, self.column);
        self.current_screen = screen_id;
        (self.row, self.column) = self.screen_cursors[self.current_screen];

        unsafe {
            self.refresh_screen();
            self.update_cursor();
        }
        true
    }

    unsafe fn refresh_screen(&mut self) {
        for (i, &entry) in self.screen_buffers[self.current_screen].iter().enumerate() {
            write_volatile(self.buffer.add(i), entry);
        }
    }

    pub unsafe fn clear_screen(&mut self) {
        let blank = vga_entry(b' ', self.color);
        for entry in self.screen_buffers[self.current_screen].iter_mut() {
            *entry = blank;
        }
        self.row = 0;
        self.column = 0;
        self.refresh_screen();
        self.update_cursor();
    }

    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }

    pub unsafe fn update_cursor(&mut self) {
        if !self.cursor_visible {
            return;
        }

        let position: u16 = (self.row * VGA_WIDTH + self.column).try_into().unwrap();

        outb(VGA_CRTC_ADDR, VGA_CURSOR_LOC_HIGH);
        outb(VGA_CRTC_DATA, (position >> 8) as u8);

        outb(VGA_CRTC_ADDR, VGA_CURSOR_LOC_LOW);
        outb(VGA_CRTC_DATA, position as u8);
    }

    pub fn enable_cursor(&mut self) {
        self.cursor_visible = true;
        unsafe {
            outb(VGA_CRTC_ADDR, 0x0A);
            outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) & 0xC0 | 0x00);

            outb(VGA_CRTC_ADDR, 0x0B);
            outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) & 0xE0 | 15);

            self.update_cursor();
        };
    }

    pub fn disable_cursor(&mut self) {
        self.cursor_visible = false;
        unsafe {
            outb(VGA_CRTC_ADDR, 0x0A);
            outb(VGA_CRTC_ADDR, 0x20);
        };
    }

    pub unsafe fn put_entry_at(&mut self, c: u8, color: u8, x: usize, y: usize) {
        if x < VGA_WIDTH && y < VGA_HEIGHT {
            let index = y * VGA_WIDTH + x;
            self.screen_buffers[self.current_screen][index] = vga_entry(c, color);
        }
    }

    pub unsafe fn scroll_up(&mut self) {
        let screen = &mut self.screen_buffers[self.current_screen];

        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let from = y * VGA_WIDTH + x;
                let to = (y - 1) * VGA_WIDTH + x;
                screen[to] = screen[from];
            }
        }

        let blank = vga_entry(b' ', self.color);
        let last_row = (VGA_HEIGHT - 1) * VGA_WIDTH;
        for x in 0..VGA_WIDTH {
            screen[last_row + x] = blank;
        }

        if self.row > 0 {
            self.row -= 1;
        }

        self.refresh_screen();
        self.update_cursor();
    }

    pub unsafe fn scroll_down(&mut self) {
        let screen = &mut self.screen_buffers[self.current_screen];

        for y in (0..VGA_HEIGHT - 1).rev() {
            for x in 0..VGA_WIDTH {
                let from = y * VGA_WIDTH + x;
                let to = (y + 1) * VGA_WIDTH + x;
                screen[to] = screen[from];
            }
        }

        let blank = vga_entry(b' ', self.color);
        let last_row = (VGA_HEIGHT - 1) * VGA_WIDTH;
        for x in 0..VGA_WIDTH {
            screen[last_row + x] = blank;
        }

        if self.row < VGA_HEIGHT - 1 {
            self.row += 1;
        }

        self.refresh_screen();
        self.update_cursor();
    }

    pub unsafe fn new_line(&mut self) {
        self.column = 0;
        self.row += 1;
        if self.row >= VGA_HEIGHT {
            self.scroll_up();
        }
        self.update_cursor();
    }

    pub unsafe fn move_cursor_left(&mut self) {
        if self.column > 0 {
            self.column -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.column = VGA_WIDTH - 1;
        }
        self.update_cursor();
    }

    pub unsafe fn move_cursor_right(&mut self) {
        if self.column < VGA_WIDTH - 1 {
            self.column += 1;
        } else if self.row < VGA_HEIGHT - 1 {
            self.row += 1;
            self.column = 0;
        }
        self.update_cursor();
    }

    pub unsafe fn move_cursor_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.update_cursor();
        } else {
            self.scroll_down();
        }
    }

    pub unsafe fn move_cursor_down(&mut self) {
        if self.row < VGA_HEIGHT - 1 {
            self.row += 1;
            self.update_cursor();
        } else {
            self.scroll_up();
            self.update_cursor();
        }
    }

    pub unsafe fn backspace(&mut self) {
        if self.column > 0 {
            self.column -= 1;
            self.put_entry_at(b' ', self.color, self.column, self.row);
        } else if self.row > 0 {
            self.row -= 1;
            self.column = VGA_WIDTH - 1;

            while self.column > 0 {
                let index = self.row * VGA_WIDTH + self.column - 1;
                let entry = read_volatile(self.buffer.add(index));
                if (entry & 0xFF) as u8 != b' ' {
                    break;
                }
                self.column -= 1;
            }
            self.put_entry_at(b' ', self.color, self.column, self.row);
        }
        self.update_cursor();
    }

    pub unsafe fn put_char(&mut self, c: u8) {
        match c {
            b'\n' => self.new_line(),
            b'\r' => {
                self.column = 0;
                self.update_cursor();
            }
            b'\t' => {
                let tab_size = 4 - (self.column % 4);
                for _ in 0..tab_size {
                    self.put_char(b' ');
                }
            }
            b'\x08' => self.backspace(),
            byte => {
                self.put_entry_at(byte, self.color, self.column, self.row);
                self.column += 1;
                if self.column >= VGA_WIDTH {
                    self.new_line();
                }
                self.update_cursor();
            }
        }
        self.refresh_screen();
    }

    pub unsafe fn write(&mut self, data: &[u8]) {
        for &byte in data {
            self.put_char(byte);
        }
    }

    pub unsafe fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
    }

    pub unsafe fn print(&mut self, msg: &str, flag: LogLevel) {
        let old_color = self.color;
        if let Some(prefix) = flag.prefix() {
            self.set_color(flag.color());
            self.write_str(prefix);
        }
        self.write_str(msg);
        self.write_str("\n");
        self.set_color(old_color);
    }

    pub unsafe fn move_to_line_start(&mut self) {
        self.column = 0;
        self.update_cursor();
    }

    pub unsafe fn move_to_line_end(&mut self) {
        self.column = VGA_WIDTH - 1;
        while self.column > 0 {
            let index = self.row * VGA_WIDTH + self.column;
            let entry = read_volatile(self.buffer.add(index));
            if (entry & 0xFF) as u8 != b' ' {
                self.column += 1;
                break;
            }
            self.column -= 1;
        }
        if self.column >= VGA_WIDTH {
            self.column = VGA_WIDTH - 1;
        }
        self.update_cursor();
    }

    pub unsafe fn page_up(&mut self) {
        for _ in 0..5 {
            self.scroll_down();
        }
    }

    pub unsafe fn page_down(&mut self) {
        for _ in 0..5 {
            self.scroll_up();
        }
    }
}

const TERMINAL_INIT: Terminal = Terminal {
    row: 0,
    column: 0,
    color: vga_entry_color(VgaColor::White, VgaColor::Black),
    buffer: 0xb8000 as *mut u16,
    cursor_visible: true,
    current_screen: 0,
    screen_buffers: [[0; VGA_BUFFER_SIZE]; SCREENS_NUMBER],
    screen_cursors: [(0, 0); SCREENS_NUMBER],
};

static mut TERMINAL: Terminal = TERMINAL_INIT;

pub fn terminal() -> &'static mut Terminal {
    unsafe { &mut TERMINAL }
}

#[macro_export]
macro_rules! kprint {
    ($level:expr, $msg:expr) => {
        unsafe { $crate::terminal().print($msg, $level) }
    };
    ($level:expr, $fmt:expr, $($arg:tt)+) => {
        {
            let term = $crate::terminal();
            if let Some(prefix) = $level.prefix() {
                term.set_color($level.color());
                unsafe { term.write_str(prefix) };
            }
            let _ = core::fmt::Write::write_fmt(term, format_args!($fmt, $($arg)+));
            unsafe { term.write_str("\n") };
            term.set_color(
                $crate::vga::terminal::vga_entry_color(
                    $crate::vga::terminal::VgaColor::LightGrey,
                    $crate::vga::terminal::VgaColor::Black,
                )
            );
        }
    };
}
