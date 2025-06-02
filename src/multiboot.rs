
#[repr(C, packed)]
pub struct MultibootInfo {
    // Multiboot info version number
    flags: u32,

    // Available memory for BIOS
    mem_lower: u32,
    mem_upper: u32,

    // "root" partition
    boot_device: u32,

    // Kernel command line
    cmdline: u32,

    // Boot-module list
    mods_count: u32,
    mods_addr: u32,

    dummy: [u8; 16],

    // Memory mapping buffer
    pub mmap_length: u32,
    pub mmap_addr: u32,

    // Drive info buffer
    drives_length: u32,
    drives_addr: u32,

    // ROM configuration table
    config_table: u32,

    // Boot loader name
    pub boot_loader_name: *const u8,

    // APM table
    apm_table: u32,

    // Video
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,

    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
}

#[repr(C, packed)]
pub struct MultibootMmapEntry {
    pub size: u32,
    pub addr: u64,
    pub len: u64,
    type_: u32,
}

