use core::{convert::TryInto, ptr::{read_volatile, write_volatile}};

use crate::{inputs::io::{inb, outb}, vga::display::vga_buffer::{vga_entry, LogLevel, VgaBuffer}};

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
pub const VGA_BUFFER_SIZE: usize = VGA_WIDTH * VGA_HEIGHT;

const SCREENS_NUMBER: usize = 3;

const VGA_CRTC_ADDR: u16 = 0x3D4;
const VGA_CRTC_DATA: u16 = 0x3D5;
const VGA_CURSOR_LOC_HIGH: u8 = 0x0E;
const VGA_CURSOR_LOC_LOW: u8 = 0x0F;

pub struct Terminal {
    pub vga_buffer: VgaBuffer,
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
        let blank = vga_entry(b' ', self.vga_buffer.color);
        for screen in 0..SCREENS_NUMBER {
            for i in 0..VGA_BUFFER_SIZE {
                self.screen_buffers[screen][i] = blank;
            }
        }
        self.enable_cursor();
        self.vga_buffer.refresh_screen(&self.screen_buffers[self.current_screen]);
        self.update_cursor();
    }

    pub fn switch_screen(&mut self, screen_id: usize) -> bool {
        if screen_id >= SCREENS_NUMBER {
            return false;
        }

        self.screen_cursors[self.current_screen] = (self.vga_buffer.row, self.vga_buffer.column);
        self.current_screen = screen_id;
        (self.vga_buffer.row, self.vga_buffer.column) = self.screen_cursors[self.current_screen];

        unsafe {
            self.vga_buffer.refresh_screen(&self.screen_buffers[self.current_screen]);
            self.update_cursor();
        }
        true
    }

    pub unsafe fn clear_screen(&mut self) {
        let blank = vga_entry(b' ', self.vga_buffer.color);
        for entry in self.screen_buffers[self.current_screen].iter_mut() {
            *entry = blank;
        }
        self.vga_buffer.clear_vga_screen();
    }

    pub unsafe fn update_cursor(&mut self) {
        if !self.cursor_visible {
            return;
        }

        let position: u16 = (self.vga_buffer.row * VGA_WIDTH + self.vga_buffer.column).try_into().unwrap();

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

    pub unsafe fn new_line(&mut self) {
        self.vga_buffer.column = 0;
        self.vga_buffer.row += 1;
        if self.vga_buffer.row >= VGA_HEIGHT {
            self.vga_buffer.scroll_up(&mut self.screen_buffers[self.current_screen]);
        }
        self.update_cursor();
    }

    pub unsafe fn move_cursor_left(&mut self) {
        if self.vga_buffer.column > 0 {
            self.vga_buffer.column -= 1;
        } else if self.vga_buffer.row > 0 {
            self.vga_buffer.row -= 1;
            self.vga_buffer.column = VGA_WIDTH - 1;
        }
        self.update_cursor();
    }

    pub unsafe fn move_cursor_right(&mut self) {
        if self.vga_buffer.column < VGA_WIDTH - 1 {
            self.vga_buffer.column += 1;
        } else if self.vga_buffer.row < VGA_HEIGHT - 1 {
            self.vga_buffer.row += 1;
            self.vga_buffer.column = 0;
        }
        self.update_cursor();
    }

    pub unsafe fn move_cursor_up(&mut self) {
        if self.vga_buffer.row > 0 {
            self.vga_buffer.row -= 1;
            self.update_cursor();
        } else {
            self.scroll_down();
        }
    }

    pub unsafe fn move_cursor_down(&mut self) {
        if self.vga_buffer.row < VGA_HEIGHT - 1 {
            self.vga_buffer.row += 1;
            self.update_cursor();
        } else {
            self.scroll_up();
            self.update_cursor();
        }
    }

    pub unsafe fn backspace(&mut self) {
        if self.vga_buffer.column > 0 {
            self.vga_buffer.column -= 1;
            self.put_entry_at(b' ', self.vga_buffer.color, self.vga_buffer.column, self.vga_buffer.row);
        } else if self.vga_buffer.row > 0 {
            self.vga_buffer.row -= 1;
            self.vga_buffer.column = VGA_WIDTH - 1;

            while self.vga_buffer.column > 0 {
                let index = self.vga_buffer.row * VGA_WIDTH + self.vga_buffer.column - 1;
                let entry = read_volatile(self.vga_buffer.buffer.add(index));
                if (entry & 0xFF) as u8 != b' ' {
                    break;
                }
                self.vga_buffer.column -= 1;
            }
            self.put_entry_at(b' ', self.vga_buffer.color, self.vga_buffer.column, self.vga_buffer.row);
        }
        self.update_cursor();
    }

    pub unsafe fn put_char(&mut self, c: u8) {
        match c {
            b'\n' => self.new_line(),
            b'\r' => {
                self.vga_buffer.column = 0;
                self.update_cursor();
            }
            b'\t' => {
                let tab_size = 4 - (self.vga_buffer.column % 4);
                for _ in 0..tab_size {
                    self.put_char(b' ');
                }
            }
            b'\x08' => self.backspace(),
            byte => {
                self.put_entry_at(byte, self.vga_buffer.color, self.vga_buffer.column, self.vga_buffer.row);
                self.vga_buffer.column += 1;
                if self.vga_buffer.column >= VGA_WIDTH {
                    self.new_line();
                }
                self.update_cursor();
            }
        }
        self.vga_buffer.refresh_screen(&self.screen_buffers[self.current_screen]);
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
        let old_color = self.vga_buffer.color;
        if let Some(prefix) = flag.prefix() {
            self.set_color(flag.color());
            self.write_str(prefix);
        }
        self.write_str(msg);
        self.write_str("\n");
        self.set_color(old_color);
    }

    pub unsafe fn move_to_line_start(&mut self) {
        self.vga_buffer.column = 0;
        self.update_cursor();
    }

    pub unsafe fn move_to_line_end(&mut self) {
        self.vga_buffer.column = VGA_WIDTH - 1;
        while self.vga_buffer.column > 0 {
            let index = self.vga_buffer.row * VGA_WIDTH + self.vga_buffer.column;
            let entry = read_volatile(self.vga_buffer.buffer.add(index));
            if (entry & 0xFF) as u8 != b' ' {
                self.vga_buffer.column += 1;
                break;
            }
            self.vga_buffer.column -= 1;
        }
        if self.vga_buffer.column >= VGA_WIDTH {
            self.vga_buffer.column = VGA_WIDTH - 1;
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
    vga_buffer: VgaBuffer::new(),
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
                term.set_color($level.vga_buffer.color());
                unsafe { term.write_str(prefix) };
            }
            let _ = core::fmt::Write::write_fmt(term, format_args!($fmt, $($arg)+));
            unsafe { term.write_str("\n") };
            term.set_color(
                $crate::vga::display::vga_buffer::vga_entry_color(
                    $crate::vga::display::vga_buffer::VgaColor::LightGrey,
                    $crate::vga::display::vga_buffer::VgaColor::Black,
                )
            );
        }
    };
}
