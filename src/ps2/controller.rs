use crate::inputs::io::{inb, outb};

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

const PS2_STATUS_OUTPUT_FULL: u8 = 0x01;
const PS2_STATUS_INPUT_FULL: u8 = 0x02;
const PS2_STATUS_MOUSE_DATA: u8 = 0x20;

pub struct PS2Controller {
    pub mouse_cycle: u8,
    pub mouse_packet: [u8; 3],
}

impl PS2Controller {
    pub fn new() -> Self {
        Self {
            mouse_cycle: 0,
            mouse_packet: [0; 3],
        }
    }

    pub fn has_data(&self) -> bool {
        unsafe { (inb(PS2_STATUS_PORT) & PS2_STATUS_OUTPUT_FULL) != 0 }
    }
    pub fn is_mouse_data(&self) -> bool {
        unsafe { (inb(PS2_STATUS_PORT) & PS2_STATUS_MOUSE_DATA) != 0 }
    }
    pub fn read_data(&mut self) -> u8 {
        unsafe { inb(PS2_DATA_PORT) }
    }
    pub fn wait_input_empty(&self) {
        while unsafe { (inb(PS2_STATUS_PORT) & PS2_STATUS_INPUT_FULL) != 0 } {}
    }
    pub fn send_command(&mut self, command: u8) {
        self.wait_input_empty();
        unsafe { outb(PS2_COMMAND_PORT, command); };
    }
    pub fn send_data(&mut self, data: u8) {
        self.wait_input_empty();
        unsafe { outb(PS2_DATA_PORT, data); };
    }
}
