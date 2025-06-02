const SCANCODE_A: u8 = 0x1E;
const SCANCODE_B: u8 = 0x30;
const SCANCODE_C: u8 = 0x2E;
const SCANCODE_D: u8 = 0x20;
const SCANCODE_E: u8 = 0x12;
const SCANCODE_F: u8 = 0x21;
const SCANCODE_G: u8 = 0x22;
const SCANCODE_H: u8 = 0x23;
const SCANCODE_I: u8 = 0x17;
const SCANCODE_J: u8 = 0x24;
const SCANCODE_K: u8 = 0x25;
const SCANCODE_L: u8 = 0x26;
const SCANCODE_M: u8 = 0x32;
const SCANCODE_N: u8 = 0x31;
const SCANCODE_O: u8 = 0x18;
const SCANCODE_P: u8 = 0x19;
const SCANCODE_Q: u8 = 0x10;
const SCANCODE_R: u8 = 0x13;
const SCANCODE_S: u8 = 0x1F;
const SCANCODE_T: u8 = 0x14;
const SCANCODE_U: u8 = 0x16;
const SCANCODE_V: u8 = 0x2F;
const SCANCODE_W: u8 = 0x11;
const SCANCODE_X: u8 = 0x2D;
const SCANCODE_Y: u8 = 0x15;
const SCANCODE_Z: u8 = 0x2C;
const SCANCODE_0: u8 = 0x0B;
const SCANCODE_1: u8 = 0x02;
const SCANCODE_2: u8 = 0x03;
const SCANCODE_3: u8 = 0x04;
const SCANCODE_4: u8 = 0x05;
const SCANCODE_5: u8 = 0x06;
const SCANCODE_6: u8 = 0x07;
const SCANCODE_7: u8 = 0x08;
const SCANCODE_8: u8 = 0x09;
const SCANCODE_9: u8 = 0x0A;
const SCANCODE_BACKSPACE: u8 = 0x0E;
const SCANCODE_TAB: u8 = 0x0F;
const SCANCODE_ENTER: u8 = 0x1C;
const SCANCODE_SPACE: u8 = 0x39;
const SCANCODE_ESC: u8 = 0x01;

pub struct KeyboardState {
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        }
    }

    pub fn process_scancode(&mut self, scancode: u8) -> Option<char> {
        let key_released = scancode & 0x80 != 0;
        let scancode = scancode & 0x7F;

        match scancode {
            0x2A | 0x36 => {
                self.shift_pressed = !key_released;
                return None;
            }
            0x1D => {
                self.ctrl_pressed = !key_released;
                return None;
            }
            0x38 => {
                self.alt_pressed = !key_released;
                return None;
            }
            _ => {}
        }

        if key_released {
            return None;
        }

        match scancode {
            SCANCODE_A => Some(if self.shift_pressed { 'A' } else { 'a' }),
            SCANCODE_B => Some(if self.shift_pressed { 'B' } else { 'b' }),
            SCANCODE_C => Some(if self.shift_pressed { 'C' } else { 'c' }),
            SCANCODE_D => Some(if self.shift_pressed { 'D' } else { 'd' }),
            SCANCODE_E => Some(if self.shift_pressed { 'E' } else { 'e' }),
            SCANCODE_F => Some(if self.shift_pressed { 'F' } else { 'f' }),
            SCANCODE_G => Some(if self.shift_pressed { 'G' } else { 'g' }),
            SCANCODE_H => Some(if self.shift_pressed { 'H' } else { 'h' }),
            SCANCODE_I => Some(if self.shift_pressed { 'I' } else { 'i' }),
            SCANCODE_J => Some(if self.shift_pressed { 'J' } else { 'j' }),
            SCANCODE_K => Some(if self.shift_pressed { 'K' } else { 'k' }),
            SCANCODE_L => Some(if self.shift_pressed { 'L' } else { 'l' }),
            SCANCODE_M => Some(if self.shift_pressed { 'M' } else { 'm' }),
            SCANCODE_N => Some(if self.shift_pressed { 'N' } else { 'n' }),
            SCANCODE_O => Some(if self.shift_pressed { 'O' } else { 'o' }),
            SCANCODE_P => Some(if self.shift_pressed { 'P' } else { 'p' }),
            SCANCODE_Q => Some(if self.shift_pressed { 'Q' } else { 'q' }),
            SCANCODE_R => Some(if self.shift_pressed { 'R' } else { 'r' }),
            SCANCODE_S => Some(if self.shift_pressed { 'S' } else { 's' }),
            SCANCODE_T => Some(if self.shift_pressed { 'T' } else { 't' }),
            SCANCODE_U => Some(if self.shift_pressed { 'U' } else { 'u' }),
            SCANCODE_V => Some(if self.shift_pressed { 'V' } else { 'v' }),
            SCANCODE_W => Some(if self.shift_pressed { 'W' } else { 'w' }),
            SCANCODE_X => Some(if self.shift_pressed { 'X' } else { 'x' }),
            SCANCODE_Y => Some(if self.shift_pressed { 'Y' } else { 'y' }),
            SCANCODE_Z => Some(if self.shift_pressed { 'Z' } else { 'z' }),
            SCANCODE_0 => Some(if self.shift_pressed { ')' } else { '0' }),
            SCANCODE_1 => Some(if self.shift_pressed { '!' } else { '1' }),
            SCANCODE_2 => Some(if self.shift_pressed { '@' } else { '2' }),
            SCANCODE_3 => Some(if self.shift_pressed { '#' } else { '3' }),
            SCANCODE_4 => Some(if self.shift_pressed { '$' } else { '4' }),
            SCANCODE_5 => Some(if self.shift_pressed { '%' } else { '5' }),
            SCANCODE_6 => Some(if self.shift_pressed { '^' } else { '6' }),
            SCANCODE_7 => Some(if self.shift_pressed { '&' } else { '7' }),
            SCANCODE_8 => Some(if self.shift_pressed { '*' } else { '8' }),
            SCANCODE_9 => Some(if self.shift_pressed { '(' } else { '9' }),
            SCANCODE_BACKSPACE => Some('\x08'),
            SCANCODE_TAB => Some('\t'),
            SCANCODE_ENTER => Some('\n'),
            SCANCODE_SPACE => Some(' '),
            SCANCODE_ESC => Some('\x1B'),
            _ => None,
        }
    }
}