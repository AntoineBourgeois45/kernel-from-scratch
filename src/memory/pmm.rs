use core::ptr::write_volatile;

use crate::memory::paging::PAGE_SIZE;

const TOTAL_MEMORY_MB: usize = 64;
pub const TOTAL_MEMORY_BYTES: usize = TOTAL_MEMORY_MB * 1024 * 1024;
pub const TOTAL_FRAMES: usize = TOTAL_MEMORY_BYTES / PAGE_SIZE;
const BITMAP_SIZE: usize = (TOTAL_FRAMES + 31) / 32;

const RESERVED_END: usize = 0x0040_0000;

static mut BITMAP: [u32; BITMAP_SIZE] = [0; BITMAP_SIZE];

#[derive(Clone, Copy)]
pub struct PmmStats {
    pub total_frames: usize,
    pub used_frames: usize,
    pub free_frames: usize,
}

pub fn init() {
    unsafe {
        for entry in BITMAP.iter_mut() {
            write_volatile(entry, 0);
        }
        mark_range_used(0, RESERVED_END);
    }
}

pub fn alloc_frame() -> Option<u32> {
    unsafe {
        for (i, entry) in BITMAP.iter_mut().enumerate() {
            if *entry != 0xFFFF_FFFF {
                for bit in 0..32 {
                    let mask = 1u32 << bit;
                    if (*entry & mask) == 0 {
                        *entry |= mask;
                        let frame_index = i * 32 + bit;
                        let addr = frame_index * PAGE_SIZE;
                        return Some(addr as u32);
                    }
                }
            }
        }
    }
    None
}

pub fn alloc_frame_below(limit: usize) -> Option<u32> {
    let max_frame = limit / PAGE_SIZE;
    unsafe {
        for (i, entry) in BITMAP.iter_mut().enumerate() {
            let base_frame = i * 32;
            if base_frame >= max_frame {
                break;
            }
            if *entry != 0xFFFF_FFFF {
                for bit in 0..32 {
                    let frame_index = base_frame + bit;
                    if frame_index >= max_frame {
                        break;
                    }
                    let mask = 1u32 << bit;
                    if (*entry & mask) == 0 {
                        *entry |= mask;
                        let addr = frame_index * PAGE_SIZE;
                        return Some(addr as u32);
                    }
                }
            }
        }
    }
    None
}

pub fn free_frame(addr: u32) {
    let frame = (addr as usize) / PAGE_SIZE;
    let index = frame / 32;
    let bit = frame % 32;
    unsafe {
        BITMAP[index] &= !(1u32 << bit);
    }
}

pub fn mark_range_used(start: usize, end: usize) {
    let mut addr = start & !(PAGE_SIZE - 1);
    while addr < end {
        let frame = addr / PAGE_SIZE;
        let index = frame / 32;
        let bit = frame % 32;
        unsafe {
            BITMAP[index] |= 1u32 << bit;
        }
        addr += PAGE_SIZE;
    }
}

pub fn stats() -> PmmStats {
    let mut used = 0usize;
    unsafe {
        for entry in BITMAP.iter() {
            used += entry.count_ones() as usize;
        }
    }
    let total = TOTAL_FRAMES;
    let free = total.saturating_sub(used);
    PmmStats {
        total_frames: total,
        used_frames: used,
        free_frames: free,
    }
}

pub fn is_frame_used(frame: usize) -> bool {
    if frame >= TOTAL_FRAMES {
        return false;
    }
    let index = frame / 32;
    let bit = frame % 32;
    unsafe { (BITMAP[index] & (1u32 << bit)) != 0 }
}
