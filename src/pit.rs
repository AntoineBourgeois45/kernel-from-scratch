use crate::inputs::io::outb;

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_BASE_FREQUENCY: u32 = 1_193_182;
const CHANNEL_0_RATE_GENERATOR: u8 = 0x34;

pub const TIMER_HZ: u32 = 100;

pub unsafe fn initialize(frequency: u32) {
    let frequency = frequency.max(19).min(PIT_BASE_FREQUENCY);
    let divisor = (PIT_BASE_FREQUENCY / frequency) as u16;

    outb(PIT_COMMAND, CHANNEL_0_RATE_GENERATOR);
    outb(PIT_CHANNEL_0, divisor as u8);
    outb(PIT_CHANNEL_0, (divisor >> 8) as u8);
}
