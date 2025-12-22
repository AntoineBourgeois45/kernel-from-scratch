use core::arch::asm;
use core::ptr::{write_volatile, read_volatile};

use crate::memory::pmm;

pub const PAGE_SIZE: usize = 4096;
pub const FLAG_PRESENT: u32 = 1 << 0;
pub const FLAG_RW: u32 = 1 << 1;
pub const FLAG_USER: u32 = 1 << 2;

const IDENTITY_MAP_MB: usize = 32;
const IDENTITY_TABLES: usize = IDENTITY_MAP_MB / 4;

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct PageTable([u32; 1024]);

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct PageDirectory([u32; 1024]);

static mut PAGE_DIRECTORY: PageDirectory = PageDirectory([0; 1024]);
static mut IDENTITY_TABLES_DATA: [PageTable; IDENTITY_TABLES] = [PageTable([0; 1024]); IDENTITY_TABLES];
static mut PAGE_TABLE_PTRS: [*mut u32; 1024] = [core::ptr::null_mut(); 1024];

#[derive(Clone, Copy)]
pub struct PagingStatus {
    pub enabled: bool,
    pub cr3: u32,
}

#[derive(Clone, Copy)]
pub struct PageInfo {
    pub dir_index: usize,
    pub table_index: usize,
    pub dir_entry: u32,
    pub table_entry: u32,
    pub present: bool,
    pub phys: Option<u32>,
}

pub unsafe fn init() {
    for i in 0..IDENTITY_TABLES {
        let table = &mut IDENTITY_TABLES_DATA[i];
        let table_phys = table as *mut PageTable as u32;
        PAGE_TABLE_PTRS[i] = table.0.as_mut_ptr();
        for j in 0..1024 {
            let addr = (i * 1024 * PAGE_SIZE + j * PAGE_SIZE) as u32;
            write_volatile(
                table.0.as_mut_ptr().add(j),
                addr | FLAG_PRESENT | FLAG_RW,
            );
        }
        write_volatile(
            PAGE_DIRECTORY.0.as_mut_ptr().add(i),
            table_phys | FLAG_PRESENT | FLAG_RW,
        );
    }
    load_directory(PAGE_DIRECTORY.0.as_mut_ptr() as u32);
    enable_paging();
}

pub unsafe fn map_page(virt: u32, phys: u32, flags: u32) {
    let entry = get_page_entry(virt, true, flags);
    write_volatile(entry, (phys & 0xFFFFF000) | FLAG_PRESENT | flags);
}

pub unsafe fn translate(virt: u32) -> Option<u32> {
    let entry = get_page_entry(virt, false, 0);
    if entry.is_null() {
        return None;
    }
    let val = read_volatile(entry);
    if val & FLAG_PRESENT == 0 {
        return None;
    }
    Some((val & 0xFFFFF000) | (virt & 0xFFF))
}

pub unsafe fn page_info(virt: u32) -> PageInfo {
    let dir_index = (virt >> 22) as usize;
    let table_index = ((virt >> 12) & 0x03FF) as usize;
    let dir_entry = read_volatile(PAGE_DIRECTORY.0.as_ptr().add(dir_index));
    if dir_entry & FLAG_PRESENT == 0 {
        return PageInfo {
            dir_index,
            table_index,
            dir_entry,
            table_entry: 0,
            present: false,
            phys: None,
        };
    }
    let mut table_ptr = PAGE_TABLE_PTRS[dir_index];
    if table_ptr.is_null() {
        table_ptr = (dir_entry & 0xFFFFF000) as *mut u32;
    }
    if table_ptr.is_null() {
        return PageInfo {
            dir_index,
            table_index,
            dir_entry,
            table_entry: 0,
            present: false,
            phys: None,
        };
    }
    let table_entry = read_volatile(table_ptr.add(table_index));
    let present = (table_entry & FLAG_PRESENT) != 0;
    let phys = if present {
        Some((table_entry & 0xFFFFF000) | (virt & 0xFFF))
    } else {
        None
    };
    PageInfo {
        dir_index,
        table_index,
        dir_entry,
        table_entry,
        present,
        phys,
    }
}

pub unsafe fn get_page(virt: u32, create: bool, flags: u32) -> *mut u32 {
    get_page_entry(virt, create, flags)
}

pub unsafe fn get_page_entry(virt: u32, create: bool, flags: u32) -> *mut u32 {
    let dir_index = (virt >> 22) as usize;
    let table_index = ((virt >> 12) & 0x03FF) as usize;
    let dir_entry = read_volatile(PAGE_DIRECTORY.0.as_ptr().add(dir_index));
    if dir_entry & FLAG_PRESENT == 0 {
        if !create {
            return core::ptr::null_mut();
        }
        let table_phys = match pmm::alloc_frame_below(IDENTITY_MAP_MB * 1024 * 1024) {
            Some(frame) => frame,
            None => return core::ptr::null_mut(),
        };
        let table_ptr = table_phys as *mut u32;
        for i in 0..1024 {
            write_volatile(table_ptr.add(i), 0);
        }
        write_volatile(
            PAGE_DIRECTORY.0.as_mut_ptr().add(dir_index),
            (table_phys & 0xFFFFF000) | FLAG_PRESENT | flags,
        );
        PAGE_TABLE_PTRS[dir_index] = table_ptr;
    }
    let table_ptr = PAGE_TABLE_PTRS[dir_index];
    table_ptr.add(table_index)
}

pub unsafe fn load_directory(phys: u32) {
    asm!("mov cr3, {}", in(reg) phys, options(nostack, preserves_flags));
}

pub unsafe fn enable_paging() {
    let mut cr0: u32;
    asm!("mov {}, cr0", out(reg) cr0, options(nostack, preserves_flags));
    cr0 |= 0x8000_0000;
    asm!("mov cr0, {}", in(reg) cr0, options(nostack, preserves_flags));
}

pub fn status() -> PagingStatus {
    unsafe {
        let mut cr0: u32;
        let mut cr3: u32;
        asm!("mov {}, cr0", out(reg) cr0, options(nostack, preserves_flags));
        asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
        PagingStatus {
            enabled: (cr0 & 0x8000_0000) != 0,
            cr3,
        }
    }
}
