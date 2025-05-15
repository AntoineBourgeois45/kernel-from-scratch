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

    fn _base(&self) -> u32 {
        ((self.base_vhi as u32) << 24) | ((self.base_hi as u32) << 16) | (self.base_lo as u32)
    }
    fn _limit(&self) -> u32 {
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
    // pub user_code_segment_selector: SegmentDescriptor,
    // pub user_data_segment_selector: SegmentDescriptor,
}

#[link_section = ".data"]
#[no_mangle]
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable {
    null_segment_selector: SegmentDescriptor::new(0, 0, 0),
    unused_segment_selector: SegmentDescriptor::new(0, 0, 0),
    kernel_code_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0x9A),
    kernel_data_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0x92),
    // user_code_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0xFA),
    // user_data_segment_selector: SegmentDescriptor::new(0, 0xFFFFFFFF, 0xF2),
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

#[repr(C)]
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
        kprint!(LogLevel::Debug, "Null Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.null_segment_selector._base(), GDT.null_segment_selector._limit(), GDT.null_segment_selector.type_);
        kprint!(LogLevel::Debug, "Kernel Code Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.kernel_code_segment_selector._base(), GDT.kernel_code_segment_selector._limit(), GDT.kernel_code_segment_selector.type_);
        kprint!(LogLevel::Debug, "Kernel Data Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.kernel_data_segment_selector._base(), GDT.kernel_data_segment_selector._limit(), GDT.kernel_data_segment_selector.type_);
        // kprint!(LogLevel::Debug, "User Code Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.user_code_segment_selector._base(), GDT.user_code_segment_selector._limit(), GDT.user_code_segment_selector.type_);
        // kprint!(LogLevel::Debug, "User Data Segment: base = {:#X}, limit = {:#X}, access_byte = {:#X}\n", GDT.user_data_segment_selector._base(), GDT.user_data_segment_selector._limit(), GDT.user_data_segment_selector.type_);
    }
}

// pub fn init_gdt() {
//     unsafe {
//         GDTR.limit = (core::mem::size_of::<GlobalDescriptorTable>() - 1) as u16;
//         GDTR.base = &GDT as *const _ as u32;

//         asm!(
//             "lgdt [{}]",
//             in(reg) &GDTR
//         );
//     }
//     kprint!(LogLevel::Info, "GDT initialized successfully\n");
// }


// Structure GDT en Rust
#[derive(Debug, Clone, Copy)]
pub struct GDT {
    pub limit: u32,
    pub base: u32,
    pub access_byte: u8,
    pub flags: u8,
}

pub fn encode_gdt_entry(target: &mut [u8], source: GDT) {
    // Vérifier que la limite peut être encodée
    if source.limit > 0xFFFFF {
        panic!("GDT cannot encode limits larger than 0xFFFFF");
    }
    
    // Encoder la limite
    target[0] = (source.limit & 0xFF) as u8;
    target[1] = ((source.limit >> 8) & 0xFF) as u8;
    target[6] = ((source.limit >> 16) & 0x0F) as u8;
    
    // Encoder la base
    target[2] = (source.base & 0xFF) as u8;
    target[3] = ((source.base >> 8) & 0xFF) as u8;
    target[4] = ((source.base >> 16) & 0xFF) as u8;
    target[7] = ((source.base >> 24) & 0xFF) as u8;
    
    // Encoder l'octet d'accès
    target[5] = source.access_byte;
    
    // Encoder les flags
    target[6] |= source.flags << 4;
}

extern "C" {
    fn setGdt(limit: u16, base: *const u8);
    fn reloadSegments();
}

// Fonction Rust qui encapsule l'appel à la fonction assembleur
pub fn load_gdt(gdt_table: &[u8]) {
    let limit = (gdt_table.len() - 1) as u16;
    let base = gdt_table.as_ptr();
    
    unsafe {
        setGdt(limit, base);
    }
}

pub fn init_gdt() {
    // Créer un tableau pour stocker l'entrée GDT (8 octets par entrée)
    let mut gdt_table = [0u8; 4 * 8];
    
    // Définir une entrée GDT
    let null_entry = GDT {
        limit: 0,           // Limite maximale
        base: 0x0,                // Base à 0
        access_byte: 0,        // Code segment, readable, present
        flags: 0xC,               // 32-bit, 4KB granularity
    };
    encode_gdt_entry(&mut gdt_table[0..8], null_entry);

    let unused_entry = GDT {
        limit: 0,           // Limite maximale
        base: 0x0,                // Base à 0
        access_byte: 0,        // Code segment, readable, present
        flags: 0xC,               // 32-bit, 4KB granularity
    };
    encode_gdt_entry(&mut gdt_table[8..16], unused_entry);

    let kernel_code_entry = GDT {
        limit: 0xFFFFF,           // Limite maximale
        base: 0x0,                // Base à 0
        access_byte: 0x9A,        // Code segment, readable, present
        flags: 0xC,               // 32-bit, 4KB granularity
    };
    encode_gdt_entry(&mut gdt_table[16..24], kernel_code_entry);

    let kernel_data_entry = GDT {
        limit: 0xFFFFF,           // Limite maximale
        base: 0x0,                // Base à 0
        access_byte: 0x92,        // Data segment, writable, present
        flags: 0xC,               // 32-bit, 4KB granularity
    };
    encode_gdt_entry(&mut gdt_table[24..32], kernel_data_entry);
    
    load_gdt(&gdt_table);

    unsafe {
        reloadSegments();
    }
    kprint!(LogLevel::Info, "GDT initialized successfully\n");
}
