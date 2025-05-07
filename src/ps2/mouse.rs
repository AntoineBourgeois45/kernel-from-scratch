use core::arch::asm;

use crate::{interrupts::io::{inb, outb}, ps2::controller::{ps2_read_data, ps2_send_command, ps2_send_data, PS2_CMD_DISABLE_SECOND_PORT, PS2_CMD_ENABLE_SECOND_PORT, PS2_CMD_READ_CONFIG, PS2_CMD_TEST_SECOND_PORT, PS2_CMD_WRITE_CONFIG, PS2_CMD_WRITE_TO_SECOND_PORT, PS2_DATA}, vga::terminal::{Terminal, VGA_HEIGHT, VGA_WIDTH}};

const MOUSE_CMD_RESET: u8 = 0xFF;
const MOUSE_CMD_SET_DEFAULTS: u8 = 0xF6;
const MOUSE_CMD_ENABLE_DATA_REPORTING: u8 = 0xF4;

const MOUSE_RES_ACK: u8 = 0xFA;
const MOUSE_RES_SELF_TEST_PASSED: u8 = 0xAA;

pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub buttons: u8,
    packed_data: [u8; 3],
    packed_index: usize,
    old_char: u16,
    old_x: usize,
    old_y: usize,
}

impl MouseState {
    pub const fn new() -> Self {
        MouseState {
            x: 0,
            y: 0,
            buttons: 0,
            packed_data: [0; 3],
            packed_index: 0,
            old_char: 0,
            old_x: 0,
            old_y: 0,
        }
    }
}

pub static mut MOUSE_STATE: MouseState = MouseState::new();

fn mouse_send_command(command: u8) -> bool {
    for _ in 0..3 {
        ps2_send_command(PS2_CMD_WRITE_TO_SECOND_PORT);
        ps2_send_data(command);

        if ps2_read_data() == MOUSE_RES_ACK {
            return true;
        }
    }
    false
}

pub fn init_mouse() -> bool {
    unsafe { asm!("cli", options(nomem, nostack)) }

    ps2_send_command(PS2_CMD_DISABLE_SECOND_PORT);

    ps2_send_command(PS2_CMD_READ_CONFIG);    
    let config = ps2_read_data();

    let new_config = (config | 0x02) & 0x40;
    ps2_send_command(PS2_CMD_WRITE_CONFIG);
    ps2_send_data(new_config);

    ps2_send_command(PS2_CMD_TEST_SECOND_PORT);
    if ps2_read_data() != 0x00 {
        unsafe { asm!("sti", options(nomem, nostack)) }
        return false;
    }

    ps2_send_command(PS2_CMD_ENABLE_SECOND_PORT);

    if !mouse_send_command(MOUSE_CMD_RESET) {
        unsafe { asm!("sti", options(nomem, nostack)) }
        return false;
    }

    if ps2_read_data() != MOUSE_RES_SELF_TEST_PASSED {
        unsafe { asm!("sti", options(nomem, nostack)) }
        return false;
    }

    ps2_read_data();

    mouse_send_command(MOUSE_CMD_SET_DEFAULTS);
    mouse_send_command(MOUSE_CMD_ENABLE_DATA_REPORTING);

    unsafe {
        MOUSE_STATE.x = (VGA_WIDTH / 2) as i32;
        MOUSE_STATE.y = (VGA_HEIGHT / 2) as i32;
        MOUSE_STATE.buttons = 0;
        MOUSE_STATE.packed_index = 0;
    }

    setup_mouse_interrupt();

    unsafe { asm!("sti", options(nomem, nostack)) }

    true
}

fn setup_mouse_interrupt() {
    unsafe {
        const PIC2_DATA: u16 = 0xA1;

        let current_mask = inb(PIC2_DATA);

        outb(PIC2_DATA, current_mask & !(1 << 4));
    }
}

pub unsafe fn handle_mouse_data(data: u8, terminal: &mut Terminal) {
    match MOUSE_STATE.packed_index {
        0 => {
            if data & 0x08 != 0 {
                MOUSE_STATE.packed_data[0] = data;
                MOUSE_STATE.packed_index = 1;
            }
        }
        1 => {
            MOUSE_STATE.packed_data[1] = data;
            MOUSE_STATE.packed_index = 2;
        }
        2 => {
            MOUSE_STATE.packed_data[2] = data;
            MOUSE_STATE.packed_index = 0;

            process_mouse_packet(terminal);
        }
        _ => {
            MOUSE_STATE.packed_index = 0;
        }
    }
}

unsafe fn process_mouse_packet(terminal: &mut Terminal) {
    let packet0 = MOUSE_STATE.packed_data[0];
    let packet1 = MOUSE_STATE.packed_data[1];
    let packet2 = MOUSE_STATE.packed_data[2];

    MOUSE_STATE.buttons = packet0 & 0x07;

    let mut x_movement = packet1 as i8 as i32;
    let mut y_movement = packet2 as i8 as i32;

    if packet0 & 0x10 != 0 && x_movement > 0 {
        x_movement -= 256;
    }
    if packet0 & 0x20 != 0 && y_movement > 0 {
        y_movement -= 256;
    }

    y_movement = -y_movement;

    MOUSE_STATE.x += x_movement;
    MOUSE_STATE.y += y_movement;

    MOUSE_STATE.x = MOUSE_STATE.x.max(0).min((VGA_WIDTH - 1) as i32);
    MOUSE_STATE.y = MOUSE_STATE.y.max(0).min((VGA_HEIGHT - 1) as i32);

    draw_mouse_cursor(terminal);
}

unsafe fn draw_mouse_cursor(terminal: &mut Terminal) {
    let x = MOUSE_STATE.x as usize;
    let y = MOUSE_STATE.y as usize;

    if MOUSE_STATE.old_x < VGA_WIDTH && MOUSE_STATE.old_y < VGA_HEIGHT {
        let index = MOUSE_STATE.old_y * VGA_WIDTH + MOUSE_STATE.old_x;
        core::ptr::write_volatile((0xb8000 as *mut u16).add(index), MOUSE_STATE.old_char);
    }

    let index = y * VGA_WIDTH + x;
    MOUSE_STATE.old_char = core::ptr::read_volatile((0xb8000 as *mut u16).add(index));
    MOUSE_STATE.old_x = x;
    MOUSE_STATE.old_y = y;

    let cursor_char = if MOUSE_STATE.buttons & 0x01 != 0 {
        b'#'
    } else if MOUSE_STATE.buttons & 0x02 != 0 {
        b'@'
    } else {
        b'X'
    };

    let cursor_color = 0x4F;

    terminal.put_entry_at(cursor_char, cursor_color, x, y);
}

pub unsafe fn handle_irq12(terminal: &mut Terminal) {
    let data = inb(PS2_DATA);
    handle_mouse_data(data, terminal);
    outb(0xA0, 0x20);
    outb(0x20, 0x20);
}
