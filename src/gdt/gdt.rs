use core::arch::asm;

use crate::{kprint, vga::terminal::LogLevel};

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SegmentDescriptor {
    limit_lo: u16,
    base_lo: u16,
    base_hi: u8,
    type_: u8,
    flags_limit_hi: u8,
    base_vhi: u8
}

impl SegmentDescriptor {
    const fn new(base: u32, limit: u32, access_byte: u8) -> Self {
        let limit_lo = (limit & 0xFFFF) as u16;
        let base_lo = (base & 0xFFFF) as u16;
        let base_hi = ((base >> 16) & 0xFF) as u8;
        let type_ = access_byte;
        let flags_limit_hi = (((limit >> 16) & 0x0F) | 0xC0) as u8;
        let base_vhi = ((base >> 24) & 0xFF) as u8;

        SegmentDescriptor {
            limit_lo,
            base_lo,
            base_hi,
            type_,
            flags_limit_hi,
            base_vhi
        }
    }

    fn base(&self) -> u32 {
        ((self.base_vhi as u32) << 24) | ((self.base_hi as u32) << 16) | (self.base_lo as u32)
    }
    fn limit(&self) -> u32 {
        (((self.flags_limit_hi & 0x0F) as u32) << 16) | (self.limit_lo as u32)
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GlobalDescriptorTable {
    pub null_segment_selector: SegmentDescriptor,
    pub unused_segment_selector: SegmentDescriptor,
    pub kernel_code_segment_selector: SegmentDescriptor,
    pub kernel_data_segment_selector: SegmentDescriptor,
    pub user_code_segment_selector: SegmentDescriptor,
    pub user_data_segment_selector: SegmentDescriptor,
}

#[link_section = ".data"]
#[no_mangle]
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable {
    null_segment_selector: SegmentDescriptor::new(0, 0, 0),
    unused_segment_selector: SegmentDescriptor::new(0, 0, 0),
    kernel_code_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0x9A),
    kernel_data_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0x92),
    user_code_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0xFA),
    user_data_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0xF2),
};

impl GlobalDescriptorTable {
    pub fn kernel_code_segment_selector() -> u16 {
        2 << 3
    }
    pub fn kernel_data_segment_selector() -> u16 {
        3 << 3
    }

    pub fn user_code_segment_selector() -> u16 {
        4 << 3
    }
    pub fn user_data_segment_selector() -> u16 {
        5 << 3
    }
}

pub struct Gdtr {
    pub limit: u16,
    pub base: u32,
}

#[link_section = ".data"]
#[no_mangle]
static mut GDTR: Gdtr = Gdtr {
    limit: 0,
    base: 0,
};

fn _print_segments() {
    unsafe {
        kprint!(LogLevel::Debug, "GDT print segments : \n");
        kprint!(LogLevel::Debug, "Null Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.null_segment_selector.base(), GDT.null_segment_selector.limit(), GDT.null_segment_selector.type_);
        kprint!(LogLevel::Debug, "Kernel Code Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.kernel_code_segment_selector.base(), GDT.kernel_code_segment_selector.limit(), GDT.kernel_code_segment_selector.type_);
        kprint!(LogLevel::Debug, "Kernel Data Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.kernel_data_segment_selector.base(), GDT.kernel_data_segment_selector.limit(), GDT.kernel_data_segment_selector.type_);
        kprint!(LogLevel::Debug, "User Code Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.user_code_segment_selector.base(), GDT.user_code_segment_selector.limit(), GDT.user_code_segment_selector.type_);
        kprint!(LogLevel::Debug, "User Data Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.user_data_segment_selector.base(), GDT.user_data_segment_selector.limit(), GDT.user_data_segment_selector.type_);
    }
}

unsafe fn debug_segments() {
    let mut cs: u16 = 0;
    let mut ds: u16 = 0;
    asm!("mov {0:x}, cs", out(reg) cs);
    asm!("mov {0:x}, ds", out(reg) ds);
    kprint!(LogLevel::Debug, "Segment registers: CS={:#x}, DS={:#x}\n", cs, ds);
}

extern "C" {
    fn load_gdt(gdtr: *const Gdtr);
    fn gdt_reload_segments();
}

pub fn init_gdt() {
    unsafe {
        GDTR.limit = (core::mem::size_of::<GlobalDescriptorTable>() - 1) as u16;
        GDTR.base = &GDT as *const _ as u32;

        load_gdt(&GDTR as *const Gdtr);
        
        // gdt_reload_segments();
    }
    kprint!(LogLevel::Info, "GDT initialized successfully\n");
}
