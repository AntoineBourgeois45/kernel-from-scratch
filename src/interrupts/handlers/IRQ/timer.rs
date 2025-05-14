use crate::{kprint, vga::terminal::LogLevel};

static mut TIMER_TICKS: u32 = 0;

pub fn handle_timer() {
    unsafe {
        TIMER_TICKS += 1;
        let ticks = TIMER_TICKS;
        if ticks % 100 == 0 {
            kprint!(LogLevel::Trace, "Timer ticks: {}\n", ticks);
        }
    }
}

