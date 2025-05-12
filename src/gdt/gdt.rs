struct SegmentDescriptor {
    limit_lo: u16,
    base_lo: u16,
    base_hi: u8,
    type_: u8,
    flags_limit_hi: u8,
    base_vhi: u8
}

impl SegmentDescriptor {
    fn new(base: u32, limit: u32, type_: u8) -> Self {
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

    fn Base(&self) -> u32 {
        ((self.base_vhi as u32) << 24) | ((self.base_hi as u32) << 16) | (self.base_lo as u32)
    }
    fn Limit(&self) -> u32 {
        ((self.flags_limit_hi as u32) << 16) | (self.limit_lo as u32)
    }
}