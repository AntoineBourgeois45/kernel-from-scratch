
use crate::interrupts::pic::{inb, outb};

const PS2_STATUS: u16 = 0x64;
const PS2_CMD: u16 = 0x64;
pub const PS2_DATA: u16 = 0x60;

pub const PS2_CMD_READ_CONFIG: u8 = 0x20;
pub const PS2_CMD_WRITE_CONFIG: u8 = 0x60;
pub const PS2_CMD_DISABLE_SECOND_PORT: u8 = 0xA7;
pub const PS2_CMD_ENABLE_SECOND_PORT: u8 = 0xA8;
pub const PS2_CMD_TEST_SECOND_PORT: u8 = 0xA9;
pub const PS2_CMD_WRITE_TO_SECOND_PORT: u8 = 0xD4;

fn ps2_wait_for_input() {
    for _ in 0..100 {
        let status = unsafe { inb(PS2_STATUS) };
        if status & 0x02 != 0 {
            return;
        }
        for _ in 0..1000 {}
    }
}

fn ps2_wait_for_output() {
    for _ in 0..100 {
        let status = unsafe { inb(PS2_STATUS) };
        if status & 0x01 != 0 {
            return;
        }
        for _ in 0..1000 {}
    }
}



pub fn ps2_send_command(command: u8) {
    ps2_wait_for_input();
    unsafe { outb(PS2_CMD, command) };
}

pub fn ps2_send_data(data: u8) {
    ps2_wait_for_input();
    unsafe { outb(PS2_DATA, data) };
}

pub fn ps2_read_data() -> u8 {
    ps2_wait_for_output();
    unsafe { inb(PS2_DATA) }
}
