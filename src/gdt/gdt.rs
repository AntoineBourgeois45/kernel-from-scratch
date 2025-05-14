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
    const fn new(base: u32, limit: u32, type_: u8) -> Self {
        let limit_lo = (limit & 0xFFFF) as u16;
        let base_lo = (base & 0xFFFF) as u16;
        let base_hi = ((base >> 16) & 0xFF) as u8;
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
        ((self.flags_limit_hi as u32) << 16) | (self.limit_lo as u32)
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

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable {
    null_segment_selector: SegmentDescriptor::new(0, 0, 0),
    unused_segment_selector: SegmentDescriptor::new(0, 0, 0),
    kernel_code_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0x9A),
    kernel_data_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0x92),
    user_code_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0xFA),
    user_data_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0xF2),
};

const DESCRIPTOR_SIZE: u16 = core::mem::size_of::<SegmentDescriptor>() as u16;

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

static mut GDTR: Gdtr = Gdtr {
    limit: 0,
    base: 0,
};

pub fn init_gdt() {
    unsafe {
        GDTR.limit = (core::mem::size_of::<GlobalDescriptorTable>() - 1) as u16;
        GDTR.base = &raw const GDT as *const _ as u32;
        asm!(
            "lgdt [{}]",
            in(reg) &raw const GDTR as *const _ as u32,
            options(nostack, nomem)
        );
    }
    kprint!(LogLevel::Info, "GDT initialized successfully\n");
}