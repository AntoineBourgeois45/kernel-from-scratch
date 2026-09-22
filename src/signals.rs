use crate::cpu::InterruptGuard;

const CALLBACKS_PER_SIGNAL: usize = 4;
const IRQ_COUNT: usize = 16;
const SIGNAL_SLOTS: usize = 4 + IRQ_COUNT;
const SCHEDULED_CAPACITY: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelSignal {
    Timer,
    Keyboard,
    Software,
    Breakpoint,
    HardwareIrq(u8),
}

impl KernelSignal {
    const fn slot(self) -> Option<usize> {
        match self {
            Self::Timer => Some(0),
            Self::Keyboard => Some(1),
            Self::Software => Some(2),
            Self::Breakpoint => Some(3),
            Self::HardwareIrq(irq) if irq < IRQ_COUNT as u8 => Some(4 + irq as usize),
            Self::HardwareIrq(_) => None,
        }
    }
}

pub type SignalCallback = fn(KernelSignal);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackHandle {
    signal_slot: usize,
    callback_slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalError {
    InvalidSignal,
    CallbackTableFull,
    ScheduleFull,
    InvalidHandle,
}

#[derive(Clone, Copy)]
struct ScheduledSignal {
    active: bool,
    signal: KernelSignal,
    due_tick: u64,
}

impl ScheduledSignal {
    const EMPTY: Self = Self {
        active: false,
        signal: KernelSignal::Software,
        due_tick: 0,
    };
}

static mut CALLBACKS: [[Option<SignalCallback>; CALLBACKS_PER_SIGNAL]; SIGNAL_SLOTS] =
    [[None; CALLBACKS_PER_SIGNAL]; SIGNAL_SLOTS];
static mut SCHEDULED: [ScheduledSignal; SCHEDULED_CAPACITY] =
    [ScheduledSignal::EMPTY; SCHEDULED_CAPACITY];
static mut TICKS: u64 = 0;

pub fn register_callback(
    signal: KernelSignal,
    callback: SignalCallback,
) -> Result<CallbackHandle, SignalError> {
    let signal_slot = signal.slot().ok_or(SignalError::InvalidSignal)?;
    let _guard = InterruptGuard::new();

    unsafe {
        for callback_slot in 0..CALLBACKS_PER_SIGNAL {
            if CALLBACKS[signal_slot][callback_slot].is_none() {
                CALLBACKS[signal_slot][callback_slot] = Some(callback);
                return Ok(CallbackHandle {
                    signal_slot,
                    callback_slot,
                });
            }
        }
    }
    Err(SignalError::CallbackTableFull)
}

pub fn unregister_callback(handle: CallbackHandle) -> Result<(), SignalError> {
    if handle.signal_slot >= SIGNAL_SLOTS || handle.callback_slot >= CALLBACKS_PER_SIGNAL {
        return Err(SignalError::InvalidHandle);
    }
    let _guard = InterruptGuard::new();
    unsafe {
        if CALLBACKS[handle.signal_slot][handle.callback_slot]
            .take()
            .is_some()
        {
            Ok(())
        } else {
            Err(SignalError::InvalidHandle)
        }
    }
}

pub fn emit(signal: KernelSignal) {
    let Some(slot) = signal.slot() else { return };
    let callbacks = {
        let _guard = InterruptGuard::new();
        unsafe { CALLBACKS[slot] }
    };

    for callback in callbacks.iter().flatten() {
        callback(signal);
    }
}

/// Queues a signal for delivery after `delay_ticks` timer interrupts.
pub fn schedule_after(signal: KernelSignal, delay_ticks: u64) -> Result<(), SignalError> {
    signal.slot().ok_or(SignalError::InvalidSignal)?;
    let _guard = InterruptGuard::new();

    unsafe {
        let due_tick = TICKS.wrapping_add(delay_ticks.max(1));
        let scheduled = core::ptr::addr_of_mut!(SCHEDULED) as *mut ScheduledSignal;
        for index in 0..SCHEDULED_CAPACITY {
            let item = &mut *scheduled.add(index);
            if !item.active {
                *item = ScheduledSignal {
                    active: true,
                    signal,
                    due_tick,
                };
                return Ok(());
            }
        }
    }
    Err(SignalError::ScheduleFull)
}

pub fn ticks() -> u64 {
    let _guard = InterruptGuard::new();
    unsafe { TICKS }
}

/// Called only from IRQ0. Due signals are copied out before callbacks run so a
/// callback is free to schedule another signal without aliasing the queue.
pub(crate) fn on_timer_interrupt() {
    let mut due = [None; SCHEDULED_CAPACITY];
    let mut due_count = 0;

    unsafe {
        TICKS = TICKS.wrapping_add(1);
        let now = TICKS;
        let scheduled = core::ptr::addr_of_mut!(SCHEDULED) as *mut ScheduledSignal;
        for index in 0..SCHEDULED_CAPACITY {
            let item = &mut *scheduled.add(index);
            if item.active && now.wrapping_sub(item.due_tick) as i64 >= 0 {
                due[due_count] = Some(item.signal);
                due_count += 1;
                item.active = false;
            }
        }
    }

    emit(KernelSignal::Timer);
    for signal in due[..due_count].iter().flatten() {
        emit(*signal);
    }
}

pub fn trigger_software_interrupt() {
    unsafe { core::arch::asm!("int 0x30", options(nomem, nostack)) }
}
