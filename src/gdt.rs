use core::arch::asm;
use core::ptr::write_volatile;

const GDT_ADDRESS: usize = 0x0000_0800;

const KERNEL_CODE_SELECTOR: u16 = 0x08;
const KERNEL_DATA_SELECTOR: u16 = 0x10;
const KERNEL_STACK_SELECTOR: u16 = 0x18;
const USER_CODE_SELECTOR: u16 = 0x20;
const USER_DATA_SELECTOR: u16 = 0x28;
const USER_STACK_SELECTOR: u16 = 0x30;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u32,
}

const fn make_entry(base: u32, limit: u32, access: u8, granularity: u8) -> GdtEntry {
    let limit_low = (limit & 0xFFFF) as u16;
    let base_low = (base & 0xFFFF) as u16;
    let base_mid = ((base >> 16) & 0xFF) as u8;
    let base_high = ((base >> 24) & 0xFF) as u8;
    let gran = ((limit >> 16) & 0x0F) as u8 | (granularity & 0xF0);
    GdtEntry {
        limit_low,
        base_low,
        base_mid,
        access,
        granularity: gran,
        base_high,
    }
}

pub unsafe fn init() {
    let gdt_ptr = GDT_ADDRESS as *mut GdtEntry;

    let null = make_entry(0, 0, 0, 0);
    let kernel_code = make_entry(0, 0xFFFFF, 0x9A, 0xC0);
    let kernel_data = make_entry(0, 0xFFFFF, 0x92, 0xC0);
    let kernel_stack = make_entry(0, 0xFFFFF, 0x92, 0xC0);
    let user_code = make_entry(0, 0xFFFFF, 0xFA, 0xC0);
    let user_data = make_entry(0, 0xFFFFF, 0xF2, 0xC0);
    let user_stack = make_entry(0, 0xFFFFF, 0xF2, 0xC0);

    write_volatile(gdt_ptr.add(0), null);
    write_volatile(gdt_ptr.add(1), kernel_code);
    write_volatile(gdt_ptr.add(2), kernel_data);
    write_volatile(gdt_ptr.add(3), kernel_stack);
    write_volatile(gdt_ptr.add(4), user_code);
    write_volatile(gdt_ptr.add(5), user_data);
    write_volatile(gdt_ptr.add(6), user_stack);

    let gdtr = Gdtr {
        limit: (7 * core::mem::size_of::<GdtEntry>() - 1) as u16,
        base: GDT_ADDRESS as u32,
    };

    asm!(
        "lgdt [{0}]",
        "mov ax, {1}",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ax, {2}",
        "mov ss, ax",
        "push {3}",
        "lea eax, [2f]",
        "push eax",
        "retf",
        "2:",
        in(reg) &gdtr,
        const KERNEL_DATA_SELECTOR,
        const KERNEL_STACK_SELECTOR,
        const KERNEL_CODE_SELECTOR,
        out("eax") _,
    );
}

pub const fn kernel_stack_selector() -> u16 {
    KERNEL_STACK_SELECTOR
}

pub const fn kernel_data_selector() -> u16 {
    KERNEL_DATA_SELECTOR
}
