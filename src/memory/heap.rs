use core::ptr::{null_mut, write_volatile};

pub type GrowFn = fn(&mut Heap, usize) -> bool;

#[repr(C)]
struct BlockHeader {
    size: usize,
    free: bool,
    next: *mut BlockHeader,
}

pub struct Heap {
    start: usize,
    brk: usize,
    max: usize,
    head: *mut BlockHeader,
    grow: Option<GrowFn>,
}

#[derive(Clone, Copy)]
pub struct HeapStats {
    pub total: usize,
    pub used: usize,
    pub free: usize,
    pub blocks: usize,
    pub free_blocks: usize,
}

impl Heap {
    pub const fn new() -> Self {
        Self {
            start: 0,
            brk: 0,
            max: 0,
            head: null_mut(),
            grow: None,
        }
    }

    pub unsafe fn init(&mut self, start: usize, max: usize, grow: GrowFn) {
        self.start = align_up(start, 8);
        self.brk = self.start;
        self.max = self.start + max;
        self.grow = Some(grow);
    }

    pub fn brk(&self) -> usize {
        self.brk
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn max(&self) -> usize {
        self.max
    }

    pub fn grow(&mut self, bytes: usize) -> bool {
        if let Some(grow_fn) = self.grow {
            grow_fn(self, bytes)
        } else {
            false
        }
    }

    pub unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        let size = align_up(size, 8);
        let mut current = self.head;
        while !current.is_null() {
            if (*current).free && (*current).size >= size {
                self.split_block(current, size);
                (*current).free = false;
                return (current as *mut u8).add(header_size());
            }
            current = (*current).next;
        }
        if self.grow(size + header_size()) {
            return self.alloc(size);
        }
        null_mut()
    }

    pub unsafe fn free(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }
        let header = (ptr as *mut BlockHeader).sub(1);
        (*header).free = true;
        self.coalesce();
    }

    pub unsafe fn size(&self, ptr: *mut u8) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let header = (ptr as *mut BlockHeader).sub(1);
        (*header).size
    }

    pub fn reserve_fixed(&mut self, bytes: usize) -> bool {
        let bytes = align_up(bytes, 8);
        if self.brk + bytes > self.max {
            return false;
        }
        unsafe {
            let block = self.brk as *mut BlockHeader;
            write_volatile(
                block,
                BlockHeader {
                    size: bytes - header_size(),
                    free: true,
                    next: null_mut(),
                },
            );
            if self.head.is_null() {
                self.head = block;
            } else {
                let mut tail = self.head;
                while !(*tail).next.is_null() {
                    tail = (*tail).next;
                }
                (*tail).next = block;
            }
        }
        self.brk += bytes;
        true
    }

    pub unsafe fn stats(&self) -> HeapStats {
        let mut used = 0usize;
        let mut free = 0usize;
        let mut blocks = 0usize;
        let mut free_blocks = 0usize;
        let mut current = self.head;
        while !current.is_null() {
            blocks += 1;
            if (*current).free {
                free_blocks += 1;
                free += (*current).size;
            } else {
                used += (*current).size;
            }
            current = (*current).next;
        }
        HeapStats {
            total: self.brk.saturating_sub(self.start),
            used,
            free,
            blocks,
            free_blocks,
        }
    }

    unsafe fn split_block(&mut self, block: *mut BlockHeader, size: usize) {
        let total = (*block).size;
        let needed = size + header_size();
        if total < needed + header_size() + 8 {
            return;
        }
        let new_block = (block as *mut u8).add(header_size() + size) as *mut BlockHeader;
        write_volatile(
            new_block,
            BlockHeader {
                size: total - size - header_size(),
                free: true,
                next: (*block).next,
            },
        );
        (*block).size = size;
        (*block).next = new_block;
    }

    unsafe fn coalesce(&mut self) {
        let mut current = self.head;
        while !current.is_null() && !(*current).next.is_null() {
            let next = (*current).next;
            let current_end = current as usize + header_size() + (*current).size;
            if (*current).free && (*next).free && current_end == next as usize {
                (*current).size += header_size() + (*next).size;
                (*current).next = (*next).next;
                continue;
            }
            current = (*current).next;
        }
    }
}

pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

const fn header_size() -> usize {
    core::mem::size_of::<BlockHeader>()
}
