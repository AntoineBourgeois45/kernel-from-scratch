use crate::inputs::io::inb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    Space, Enter, Tab, Backspace, Delete, Escape,

    LeftShift, RightShift, LeftCtrl, RightCtrl, LeftAlt, RightAlt,

    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    Home, End, PageUp, PageDown, Insert,

    Semicolon, Equals, Comma, Minus, Period, Slash, Backtick,
    LeftBracket, Backslash, RightBracket, Quote,

    Unknown(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub pressed: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyEvent {
    pub fn to_char(self) -> Option<char> {
        if !self.pressed {
            return None;
        }

        match self.key {
            KeyCode::A => Some(if self.shift { 'A' } else { 'a' }),
            KeyCode::B => Some(if self.shift { 'B' } else { 'b' }),
            KeyCode::C => Some(if self.shift { 'C' } else { 'c' }),
            KeyCode::D => Some(if self.shift { 'D' } else { 'd' }),
            KeyCode::E => Some(if self.shift { 'E' } else { 'e' }),
            KeyCode::F => Some(if self.shift { 'F' } else { 'f' }),
            KeyCode::G => Some(if self.shift { 'G' } else { 'g' }),
            KeyCode::H => Some(if self.shift { 'H' } else { 'h' }),
            KeyCode::I => Some(if self.shift { 'I' } else { 'i' }),
            KeyCode::J => Some(if self.shift { 'J' } else { 'j' }),
            KeyCode::K => Some(if self.shift { 'K' } else { 'k' }),
            KeyCode::L => Some(if self.shift { 'L' } else { 'l' }),
            KeyCode::M => Some(if self.shift { 'M' } else { 'm' }),
            KeyCode::N => Some(if self.shift { 'N' } else { 'n' }),
            KeyCode::O => Some(if self.shift { 'O' } else { 'o' }),
            KeyCode::P => Some(if self.shift { 'P' } else { 'p' }),
            KeyCode::Q => Some(if self.shift { 'Q' } else { 'q' }),
            KeyCode::R => Some(if self.shift { 'R' } else { 'r' }),
            KeyCode::S => Some(if self.shift { 'S' } else { 's' }),
            KeyCode::T => Some(if self.shift { 'T' } else { 't' }),
            KeyCode::U => Some(if self.shift { 'U' } else { 'u' }),
            KeyCode::V => Some(if self.shift { 'V' } else { 'v' }),
            KeyCode::W => Some(if self.shift { 'W' } else { 'w' }),
            KeyCode::X => Some(if self.shift { 'X' } else { 'x' }),
            KeyCode::Y => Some(if self.shift { 'Y' } else { 'y' }),
            KeyCode::Z => Some(if self.shift { 'Z' } else { 'z' }),

            KeyCode::Num0 => Some(if self.shift { ')' } else { '0' }),
            KeyCode::Num1 => Some(if self.shift { '!' } else { '1' }),
            KeyCode::Num2 => Some(if self.shift { '@' } else { '2' }),
            KeyCode::Num3 => Some(if self.shift { '#' } else { '3' }),
            KeyCode::Num4 => Some(if self.shift { '$' } else { '4' }),
            KeyCode::Num5 => Some(if self.shift { '%' } else { '5' }),
            KeyCode::Num6 => Some(if self.shift { '^' } else { '6' }),
            KeyCode::Num7 => Some(if self.shift { '&' } else { '7' }),
            KeyCode::Num8 => Some(if self.shift { '*' } else { '8' }),
            KeyCode::Num9 => Some(if self.shift { '(' } else { '9' }),

            KeyCode::Space => Some(' '),
            KeyCode::Enter => Some('\n'),
            KeyCode::Tab => Some('\t'),
            KeyCode::Backspace => Some('\x08'),

            KeyCode::Semicolon => Some(if self.shift { ':' } else { ';' }),
            KeyCode::Equals => Some(if self.shift { '+' } else { '=' }),
            KeyCode::Comma => Some(if self.shift { '<' } else { ',' }),
            KeyCode::Minus => Some(if self.shift { '_' } else { '-' }),
            KeyCode::Period => Some(if self.shift { '>' } else { '.' }),
            KeyCode::Slash => Some(if self.shift { '?' } else { '/' }),
            KeyCode::Backtick => Some(if self.shift { '~' } else { '`' }),
            KeyCode::LeftBracket => Some(if self.shift { '{' } else { '[' }),
            KeyCode::Backslash => Some(if self.shift { '|' } else { '\\' }),
            KeyCode::RightBracket => Some(if self.shift { '}' } else { ']' }),
            KeyCode::Quote => Some(if self.shift { '"' } else { '\'' }),
            
            _ => None,
        }
    }
}

pub struct KeyboardState {
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
    pub extended_mode: bool,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            extended_mode: false,
        }
    }

    pub fn process_scancode(&mut self, scancode: u8) -> Option<KeyEvent> {
        if scancode == 0xE0 {
            self.extended_mode = true;
            return None;
        }

        let key_released = scancode & 0x80 != 0;
        let scancode = scancode & 0x7F;

        let key = if self.extended_mode {
            self.extended_mode = false;
            match scancode {
                0x48 => KeyCode::ArrowUp,
                0x50 => KeyCode::ArrowDown,
                0x4B => KeyCode::ArrowLeft,
                0x4D => KeyCode::ArrowRight,
                0x47 => KeyCode::Home,
                0x4F => KeyCode::End,
                0x49 => KeyCode::PageUp,
                0x51 => KeyCode::PageDown,
                0x52 => KeyCode::Insert,
                0x53 => KeyCode::Delete,
                0x1D => KeyCode::RightCtrl,
                0x38 => KeyCode::RightAlt,
                _ => KeyCode::Unknown(scancode),
            }
        } else {
            match scancode {
                0x1E => KeyCode::A,
                0x30 => KeyCode::B,
                0x2E => KeyCode::C,
                0x20 => KeyCode::D,
                0x12 => KeyCode::E,
                0x21 => KeyCode::F,
                0x22 => KeyCode::G,
                0x23 => KeyCode::H,
                0x17 => KeyCode::I,
                0x24 => KeyCode::J,
                0x25 => KeyCode::K,
                0x26 => KeyCode::L,
                0x32 => KeyCode::M,
                0x31 => KeyCode::N,
                0x18 => KeyCode::O,
                0x19 => KeyCode::P,
                0x10 => KeyCode::Q,
                0x13 => KeyCode::R,
                0x1F => KeyCode::S,
                0x14 => KeyCode::T,
                0x16 => KeyCode::U,
                0x2F => KeyCode::V,
                0x11 => KeyCode::W,
                0x2D => KeyCode::X,
                0x15 => KeyCode::Y,
                0x2C => KeyCode::Z,

                0x0B => KeyCode::Num0,
                0x02 => KeyCode::Num1,
                0x03 => KeyCode::Num2,
                0x04 => KeyCode::Num3,
                0x05 => KeyCode::Num4,
                0x06 => KeyCode::Num5,
                0x07 => KeyCode::Num6,
                0x08 => KeyCode::Num7,
                0x09 => KeyCode::Num8,
                0x0A => KeyCode::Num9,

                0x39 => KeyCode::Space,
                0x1C => KeyCode::Enter,
                0x0F => KeyCode::Tab,
                0x0E => KeyCode::Backspace,
                0x01 => KeyCode::Escape,

                0x2A => KeyCode::LeftShift,
                0x36 => KeyCode::RightShift,
                0x1D => KeyCode::LeftCtrl,
                0x38 => KeyCode::LeftAlt,

                0x3B => KeyCode::F1,
                0x3C => KeyCode::F2,
                0x3D => KeyCode::F3,
                0x3E => KeyCode::F4,
                0x3F => KeyCode::F5,
                0x40 => KeyCode::F6,
                0x41 => KeyCode::F7,
                0x42 => KeyCode::F8,
                0x43 => KeyCode::F9,
                0x44 => KeyCode::F10,
                0x57 => KeyCode::F11,
                0x58 => KeyCode::F12,

                0x27 => KeyCode::Semicolon,
                0x0D => KeyCode::Equals,
                0x33 => KeyCode::Comma,
                0x0C => KeyCode::Minus,
                0x34 => KeyCode::Period,
                0x35 => KeyCode::Slash,
                0x29 => KeyCode::Backtick,
                0x1A => KeyCode::LeftBracket,
                0x2B => KeyCode::Backslash,
                0x1B => KeyCode::RightBracket,
                0x28 => KeyCode::Quote,

                _ => KeyCode::Unknown(scancode),
            }
        };

        match key {
            KeyCode::LeftShift | KeyCode::RightShift => {
                self.shift_pressed = !key_released;
            }
            KeyCode::LeftCtrl | KeyCode::RightCtrl => {
                self.ctrl_pressed = !key_released;
            }
            KeyCode::LeftAlt | KeyCode::RightAlt => {
                self.alt_pressed = !key_released;
            }
            _ => {}
        }

        Some(KeyEvent {
            key,
            pressed: !key_released,
            shift: self.shift_pressed,
            ctrl: self.ctrl_pressed,
            alt: self.alt_pressed,
        })
    }
}

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_STATUS_OUTPUT_FULL: u8 = 0x01;

pub fn keyboard_has_data() -> bool {
    unsafe { (inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL) != 0 }
}

pub fn keyboard_read_scancode() -> u8 {
    unsafe { inb(PS2_DATA_PORT) }
}
