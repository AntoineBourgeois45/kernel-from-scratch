use crate::{kprint, vga::terminal::LogLevel};

extern "C" {
    fn setGdt(limit: u16, base: *const u8);
    fn reloadSegments();
}

#[derive(Debug, Clone, Copy)]
pub struct GDT {
    pub limit: u32,
    pub base: u32,
    pub access_byte: u8,
    pub flags: u8,
}

pub fn encode_gdt_entry(target: &mut [u8], source: GDT) {
    if source.limit > 0xFFFFF {
        kprint!(LogLevel::Error, "GDT cannot encode limits larger than 0xFFFFF");
    }
    
    target[0] = (source.limit & 0xFF) as u8;
    target[1] = ((source.limit >> 8) & 0xFF) as u8;
    target[6] = ((source.limit >> 16) & 0x0F) as u8;
    
    target[2] = (source.base & 0xFF) as u8;
    target[3] = ((source.base >> 8) & 0xFF) as u8;
    target[4] = ((source.base >> 16) & 0xFF) as u8;
    target[7] = ((source.base >> 24) & 0xFF) as u8;
    
    target[5] = source.access_byte;
    
    target[6] |= source.flags << 4;
}

pub fn load_gdt(gdt_table: &[u8]) {
    let limit = (gdt_table.len() - 1) as u16;
    let base = gdt_table.as_ptr();
        
    unsafe {
        setGdt(limit, base);
    }
}

pub fn init_gdt() {
    let mut gdt_table = [0u8; 6 * 8];
    
    let null_entry = GDT {
        limit: 0,
        base: 0x0,
        access_byte: 0,
        flags: 0xC,
    };
    encode_gdt_entry(&mut gdt_table[0..8], null_entry);

    let unused_entry = GDT {
        limit: 0,
        base: 0x0,
        access_byte: 0,
        flags: 0xC,
    };
    encode_gdt_entry(&mut gdt_table[8..16], unused_entry);

    let kernel_code_entry = GDT {
        limit: 0xFFFFF,
        base: 0x0,
        access_byte: 0x9A,
        flags: 0xC,
    };
    encode_gdt_entry(&mut gdt_table[16..24], kernel_code_entry);

    let kernel_data_entry = GDT {
        limit: 0xFFFFF,
        base: 0x0,
        access_byte: 0x92,
        flags: 0xC,
    };
    encode_gdt_entry(&mut gdt_table[24..32], kernel_data_entry);
    
    let user_code_entry = GDT {
        limit: 0xFFFFF,
        base: 0x0,
        access_byte: 0xFA,
        flags: 0xC,
    };
    encode_gdt_entry(&mut gdt_table[32..40], user_code_entry);

    let user_data_entry = GDT {
        limit: 0xFFFFF,
        base: 0x0,
        access_byte: 0xF2,
        flags: 0xC,
    };
    encode_gdt_entry(&mut gdt_table[40..48], user_data_entry);
    
    load_gdt(&gdt_table);

    unsafe {
        reloadSegments();
    }
    kprint!(LogLevel::Info, "GDT initialized successfully\n");
}
