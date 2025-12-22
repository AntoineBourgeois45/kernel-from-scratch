pub mod paging;
pub mod pmm;
pub mod heap;

use crate::kprint;
use crate::vga::terminal::LogLevel;
use heap::Heap;

pub const KERNEL_SPACE_END: u32 = 0xBFFF_FFFF;
pub const USER_SPACE_START: u32 = 0xC000_0000;

const PHYS_HEAP_START: usize = 0x0040_0000;
const PHYS_HEAP_INITIAL: usize = 0x0010_0000;
const PHYS_HEAP_MAX: usize = 0x0040_0000;

const VIRT_HEAP_START: usize = 0x4000_0000;
const VIRT_HEAP_INITIAL: usize = 0x0010_0000;
const VIRT_HEAP_MAX: usize = 0x0100_0000;

static mut KHEAP: Heap = Heap::new();
static mut VHEAP: Heap = Heap::new();

pub unsafe fn init() {
    pmm::init();
    pmm::mark_range_used(PHYS_HEAP_START, PHYS_HEAP_START + PHYS_HEAP_MAX);
    paging::init();
    init_heaps();
}

unsafe fn init_heaps() {
    KHEAP.init(PHYS_HEAP_START, PHYS_HEAP_MAX, grow_kheap);
    if !KHEAP.reserve_fixed(PHYS_HEAP_INITIAL) {
        crate::kpanic::warn("kheap init failed");
    }
    VHEAP.init(VIRT_HEAP_START, VIRT_HEAP_MAX, grow_vheap);
    if !VHEAP.grow(VIRT_HEAP_INITIAL) {
        crate::kpanic::warn("vheap init failed");
    }
}

pub unsafe fn kmalloc(size: usize) -> *mut u8 {
    let ptr = KHEAP.alloc(size);
    if ptr.is_null() {
        crate::kpanic::warn("kmalloc failed");
    }
    ptr
}

pub unsafe fn kfree(ptr: *mut u8) {
    KHEAP.free(ptr);
}

pub unsafe fn ksize(ptr: *mut u8) -> usize {
    KHEAP.size(ptr)
}

pub unsafe fn kbrk(bytes: usize) -> usize {
    if KHEAP.grow(bytes) {
        KHEAP.brk()
    } else {
        0
    }
}

pub unsafe fn vmalloc(size: usize) -> *mut u8 {
    let ptr = VHEAP.alloc(size);
    if ptr.is_null() {
        crate::kpanic::warn("vmalloc failed");
    }
    ptr
}

pub unsafe fn vfree(ptr: *mut u8) {
    VHEAP.free(ptr);
}

pub unsafe fn vsize(ptr: *mut u8) -> usize {
    VHEAP.size(ptr)
}

pub unsafe fn vbrk(bytes: usize) -> usize {
    if VHEAP.grow(bytes) {
        VHEAP.brk()
    } else {
        0
    }
}

pub unsafe fn memtest() -> bool {
    let p = kmalloc(64);
    if p.is_null() {
        return false;
    }
    for i in 0..64 {
        core::ptr::write_volatile(p.add(i), i as u8);
    }
    for i in 0..64 {
        if core::ptr::read_volatile(p.add(i)) != i as u8 {
            kfree(p);
            return false;
        }
    }
    kfree(p);

    let v = vmalloc(paging::PAGE_SIZE * 2);
    if v.is_null() {
        return false;
    }
    for i in 0..(paging::PAGE_SIZE * 2) {
        core::ptr::write_volatile(v.add(i), 0xAA);
    }
    for i in 0..(paging::PAGE_SIZE * 2) {
        if core::ptr::read_volatile(v.add(i)) != 0xAA {
            vfree(v);
            return false;
        }
    }
    vfree(v);
    true
}

pub unsafe fn print_info() {
    let paging_status = paging::status();
    let pmm_stats = pmm::stats();
    let kstats = KHEAP.stats();
    let vstats = VHEAP.stats();
    kprint!(LogLevel::Info, "=== Memory Info ===");
    kprint!(
        LogLevel::Info,
        "Paging: {} CR3=0x{:08x}",
        if paging_status.enabled { "on" } else { "off" },
        paging_status.cr3
    );
    kprint!(
        LogLevel::Info,
        "Kernel space <= 0x{:08x} | User space >= 0x{:08x}",
        KERNEL_SPACE_END,
        USER_SPACE_START
    );
    kprint!(
        LogLevel::Info,
        "PMM: total={} frames ({} MB) used={} free={}",
        pmm_stats.total_frames,
        pmm::TOTAL_MEMORY_BYTES / (1024 * 1024),
        pmm_stats.used_frames,
        pmm_stats.free_frames
    );
    kprint!(
        LogLevel::Info,
        "KHEAP: start=0x{:08x} brk=0x{:08x} max=0x{:08x}",
        KHEAP.start(),
        KHEAP.brk(),
        KHEAP.max()
    );
    kprint!(
        LogLevel::Info,
        "KHEAP: used={} free={} blocks={} free_blocks={}",
        kstats.used,
        kstats.free,
        kstats.blocks,
        kstats.free_blocks
    );
    kprint!(
        LogLevel::Info,
        "VHEAP: start=0x{:08x} brk=0x{:08x} max=0x{:08x}",
        VHEAP.start(),
        VHEAP.brk(),
        VHEAP.max()
    );
    kprint!(
        LogLevel::Info,
        "VHEAP: used={} free={} blocks={} free_blocks={}",
        vstats.used,
        vstats.free,
        vstats.blocks,
        vstats.free_blocks
    );
    match paging::translate(0x000B_8000) {
        Some(phys) => kprint!(LogLevel::Info, "Map VGA 0x000B8000 -> 0x{:08x}", phys),
        None => kprint!(LogLevel::Warning, "Map VGA 0x000B8000 -> unmapped"),
    }
    match paging::translate(VIRT_HEAP_START as u32) {
        Some(phys) => kprint!(
            LogLevel::Info,
            "Map VHEAP 0x{:08x} -> 0x{:08x}",
            VIRT_HEAP_START as u32,
            phys
        ),
        None => kprint!(
            LogLevel::Warning,
            "Map VHEAP 0x{:08x} -> unmapped",
            VIRT_HEAP_START as u32
        ),
    }
    kprint!(LogLevel::Info, "===================");
}

pub unsafe fn print_map(addr: u32) {
    let info = paging::page_info(addr);
    kprint!(
        LogLevel::Info,
        "Map 0x{:08x}: PDE[{}]=0x{:08x}",
        addr,
        info.dir_index,
        info.dir_entry
    );
    if info.dir_entry & paging::FLAG_PRESENT == 0 {
        kprint!(LogLevel::Warning, "PDE not present");
        return;
    }
    kprint!(
        LogLevel::Info,
        "PTE[{}]=0x{:08x}",
        info.table_index,
        info.table_entry
    );
    kprint!(
        LogLevel::Info,
        "Flags: P={} RW={} US={}",
        (info.table_entry & paging::FLAG_PRESENT) != 0,
        (info.table_entry & paging::FLAG_RW) != 0,
        (info.table_entry & paging::FLAG_USER) != 0
    );
    if let Some(phys) = info.phys {
        kprint!(LogLevel::Info, "Phys = 0x{:08x}", phys);
    } else {
        kprint!(LogLevel::Warning, "Page not present");
    }
}

pub unsafe fn print_pmm_range(start_frame: usize, count: usize) {
    let end = core::cmp::min(start_frame + count, pmm::TOTAL_FRAMES);
    kprint!(
        LogLevel::Info,
        "PMM frames {}..{} (total {})",
        start_frame,
        end,
        pmm::TOTAL_FRAMES
    );
    for frame in start_frame..end {
        let addr = frame * paging::PAGE_SIZE;
        let used = pmm::is_frame_used(frame);
        kprint!(
            LogLevel::Info,
            "frame {:05} addr=0x{:08x} {}",
            frame,
            addr as u32,
            if used { "used" } else { "free" }
        );
    }
}

pub unsafe fn memdump(addr: u32, len: usize) {
    let mut offset = 0usize;
    let max_len = core::cmp::min(len, 256);
    let ptr = addr as *const u8;
    while offset < max_len {
        let base = addr as usize + offset;
        let b0 = if offset < max_len {
            core::ptr::read_volatile(ptr.add(offset))
        } else {
            0
        };
        let b1 = if offset + 1 < max_len {
            core::ptr::read_volatile(ptr.add(offset + 1))
        } else {
            0
        };
        let b2 = if offset + 2 < max_len {
            core::ptr::read_volatile(ptr.add(offset + 2))
        } else {
            0
        };
        let b3 = if offset + 3 < max_len {
            core::ptr::read_volatile(ptr.add(offset + 3))
        } else {
            0
        };
        let b4 = if offset + 4 < max_len {
            core::ptr::read_volatile(ptr.add(offset + 4))
        } else {
            0
        };
        let b5 = if offset + 5 < max_len {
            core::ptr::read_volatile(ptr.add(offset + 5))
        } else {
            0
        };
        let b6 = if offset + 6 < max_len {
            core::ptr::read_volatile(ptr.add(offset + 6))
        } else {
            0
        };
        let b7 = if offset + 7 < max_len {
            core::ptr::read_volatile(ptr.add(offset + 7))
        } else {
            0
        };
        kprint!(
            LogLevel::Info,
            "0x{:08x}: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            base as u32,
            b0,
            b1,
            b2,
            b3,
            b4,
            b5,
            b6,
            b7
        );
        offset += 8;
    }
    if len > max_len {
        kprint!(
            LogLevel::Warning,
            "memdump truncated to {} bytes",
            max_len
        );
    }
}

fn grow_kheap(heap: &mut Heap, bytes: usize) -> bool {
    let needed = heap::align_up(bytes, paging::PAGE_SIZE);
    heap.reserve_fixed(needed)
}

fn grow_vheap(heap: &mut Heap, bytes: usize) -> bool {
    let needed = heap::align_up(bytes, paging::PAGE_SIZE);
    let mut addr = heap.brk();
    let end = addr + needed;
    while addr < end {
        let frame = match pmm::alloc_frame() {
            Some(frame) => frame,
            None => return false,
        };
        unsafe { paging::map_page(addr as u32, frame, paging::FLAG_RW) };
        addr += paging::PAGE_SIZE;
    }
    heap.reserve_fixed(needed)
}
