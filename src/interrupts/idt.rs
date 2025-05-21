use crate::interrupts::handlers::isr;
use crate::interrupts::pic::init_pic;
use crate::{kprint, vga::terminal::LogLevel};

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    pub isr_low: u16,
    pub kernel_cs: u16,
    pub reserved: u8,
    pub attributes: u8,
    pub isr_high: u16
}

#[repr(C, align(16))]
pub struct Idtr {
    pub limit: u16,
    pub base: u32,
}

#[repr(C, align(4096))]
struct AlignedIDT {
    entries: [IdtEntry; 256],
}

#[no_mangle]
#[link_section = ".data"]
static mut IDT: AlignedIDT = AlignedIDT {
    entries: [IdtEntry {
        isr_low: 0,
        kernel_cs: 0,
        reserved: 0,
        attributes: 0,
        isr_high: 0,
    }; 256],
};

#[no_mangle]
static mut IDTR: Idtr = Idtr {
    limit: 0,
    base: 0,
};

#[repr(C, packed)]
pub struct Registers {
    ds: u32,
    edi: u32,
    esi: u32,
    ebp: u32,
    kern_esp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,
    interrupt_number: u32,
    error_code: u32,
    eip: u32,
    cs: u32,
    eflags: u32,
    esp: u32,
    ss: u32,
}

pub unsafe fn set_handler(index: usize, handler_function_addr: u32, selector: u16, attributes: u8) {
    IDT.entries[index].isr_low = handler_function_addr as u16;
    IDT.entries[index].kernel_cs = selector;
    IDT.entries[index].reserved = 0;
    IDT.entries[index].attributes = attributes;
    IDT.entries[index].isr_high = (handler_function_addr >> 16) as u16;
}

extern "C" {
    fn isr0();   fn isr1();   fn isr2();   fn isr3();
    fn isr4();   fn isr5();   fn isr6();   fn isr7();
    fn isr8();   fn isr9();   fn isr10();  fn isr11();
    fn isr12();  fn isr13();  fn isr14();  fn isr15();
    fn isr16();  fn isr17();  fn isr18();  fn isr19();
    fn isr20();  fn isr21();  fn isr22();  fn isr23();
    fn isr24();  fn isr25();  fn isr26();  fn isr27();
    fn isr28();  fn isr29();  fn isr30();  fn isr31();
    fn isr32();  fn isr33();  fn isr34();  fn isr35();
    fn isr36();  fn isr37();  fn isr38();  fn isr39();
    fn isr40();  fn isr41();  fn isr42();  fn isr43();
    fn isr44();  fn isr45();  fn isr46();  fn isr47();
    fn isr48();  fn isr49();  fn isr50();  fn isr51();
    fn isr52();  fn isr53();  fn isr54();  fn isr55();
    fn isr56();  fn isr57();  fn isr58();  fn isr59();
    fn isr60();  fn isr61();  fn isr62();  fn isr63();
    fn isr64();  fn isr65();  fn isr66();  fn isr67();
    fn isr68();  fn isr69();  fn isr70();  fn isr71();
    fn isr72();  fn isr73();  fn isr74();  fn isr75();
    fn isr76();  fn isr77();  fn isr78();  fn isr79();
    fn isr80();  fn isr81();  fn isr82();  fn isr83();
    fn isr84();  fn isr85();  fn isr86();  fn isr87();
    fn isr88();  fn isr89();  fn isr90();  fn isr91();
    fn isr92();  fn isr93();  fn isr94();  fn isr95();
    fn isr96();  fn isr97();  fn isr98();  fn isr99();
    fn isr100(); fn isr101(); fn isr102(); fn isr103();
    fn isr104(); fn isr105(); fn isr106(); fn isr107();
    fn isr108(); fn isr109(); fn isr110(); fn isr111();
    fn isr112(); fn isr113(); fn isr114(); fn isr115();
    fn isr116(); fn isr117(); fn isr118(); fn isr119();
    fn isr120(); fn isr121(); fn isr122(); fn isr123();
    fn isr124(); fn isr125(); fn isr126(); fn isr127();
    fn isr128(); fn isr129(); fn isr130(); fn isr131();
    fn isr132(); fn isr133(); fn isr134(); fn isr135();
    fn isr136(); fn isr137(); fn isr138(); fn isr139();
    fn isr140(); fn isr141(); fn isr142(); fn isr143();
    fn isr144(); fn isr145(); fn isr146(); fn isr147();
    fn isr148(); fn isr149(); fn isr150(); fn isr151();
    fn isr152(); fn isr153(); fn isr154(); fn isr155();
    fn isr156(); fn isr157(); fn isr158(); fn isr159();
    fn isr160(); fn isr161(); fn isr162(); fn isr163();
    fn isr164(); fn isr165(); fn isr166(); fn isr167();
    fn isr168(); fn isr169(); fn isr170(); fn isr171();
    fn isr172(); fn isr173(); fn isr174(); fn isr175();
    fn isr176(); fn isr177(); fn isr178(); fn isr179();
    fn isr180(); fn isr181(); fn isr182(); fn isr183();
    fn isr184(); fn isr185(); fn isr186(); fn isr187();
    fn isr188(); fn isr189(); fn isr190(); fn isr191();
    fn isr192(); fn isr193(); fn isr194(); fn isr195();
    fn isr196(); fn isr197(); fn isr198(); fn isr199();
    fn isr200(); fn isr201(); fn isr202(); fn isr203();
    fn isr204(); fn isr205(); fn isr206(); fn isr207();
    fn isr208(); fn isr209(); fn isr210(); fn isr211();
    fn isr212(); fn isr213(); fn isr214(); fn isr215();
    fn isr216(); fn isr217(); fn isr218(); fn isr219();
    fn isr220(); fn isr221(); fn isr222(); fn isr223();
    fn isr224(); fn isr225(); fn isr226(); fn isr227();
    fn isr228(); fn isr229(); fn isr230(); fn isr231();
    fn isr232(); fn isr233(); fn isr234(); fn isr235();
    fn isr236(); fn isr237(); fn isr238(); fn isr239();
    fn isr240(); fn isr241(); fn isr242(); fn isr243();
    fn isr244(); fn isr245(); fn isr246(); fn isr247();
    fn isr248(); fn isr249(); fn isr250(); fn isr251();
    fn isr252(); fn isr253(); fn isr254(); fn isr255();
}

fn handle_divide_by_zero() {
    kprint!(LogLevel::Error, "Divide by zero exception\n");
}

fn handle_debug() {
    kprint!(LogLevel::Error, "Debug exception\n");
}

fn handle_unknown(int_no: u32) {
    kprint!(LogLevel::Error, "Unknown exception: {}\n", int_no);
}

#[no_mangle]
extern "C" fn exception_handler(regs: &Registers) {
    let interrupt_number = regs.interrupt_number;
    kprint!(LogLevel::Error, "Interrupt {} occurred", interrupt_number);
    match interrupt_number {
        0  => handle_divide_by_zero(),
        1  => handle_debug(),
        3  => isr::breakpoint::handle_breakpoint(),
        8  => isr::double_fault::handle_double_fault(),
        13 => isr::general_protection_fault::handle_general_protection_fault(),
        14 => isr::page_fault::handle_page_fault(),
        n  => handle_unknown(n),
    }
}

extern "C" {
    fn setIdt(limit: u16, base: *const u8);
}

pub fn init_idt() {
    unsafe {
        let handlers: [unsafe extern "C" fn() -> (); 256] = [
            isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7,
            isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15,
            isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23,
            isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31,
            isr32, isr33, isr34, isr35, isr36, isr37, isr38, isr39,
            isr40, isr41, isr42, isr43, isr44, isr45, isr46, isr47,
            isr48, isr49, isr50, isr51, isr52, isr53, isr54, isr55,
            isr56, isr57, isr58, isr59, isr60, isr61, isr62, isr63,
            isr64, isr65, isr66, isr67, isr68, isr69, isr70, isr71,
            isr72, isr73, isr74, isr75, isr76, isr77, isr78, isr79,
            isr80, isr81, isr82, isr83, isr84, isr85, isr86, isr87,
            isr88, isr89, isr90, isr91, isr92, isr93, isr94, isr95,
            isr96, isr97, isr98, isr99, isr100, isr101, isr102, isr103,
            isr104, isr105, isr106, isr107, isr108, isr109, isr110, isr111,
            isr112, isr113, isr114, isr115, isr116, isr117, isr118, isr119,
            isr120, isr121, isr122, isr123, isr124, isr125, isr126, isr127,
            isr128, isr129, isr130, isr131, isr132, isr133, isr134, isr135,
            isr136, isr137, isr138, isr139, isr140, isr141, isr142, isr143,
            isr144, isr145, isr146, isr147, isr148, isr149, isr150, isr151,
            isr152, isr153, isr154, isr155, isr156, isr157, isr158, isr159,
            isr160, isr161, isr162, isr163, isr164, isr165, isr166, isr167,
            isr168, isr169, isr170, isr171, isr172, isr173, isr174, isr175,
            isr176, isr177, isr178, isr179, isr180, isr181, isr182, isr183,
            isr184, isr185, isr186, isr187, isr188, isr189, isr190, isr191,
            isr192, isr193, isr194, isr195, isr196, isr197, isr198, isr199,
            isr200, isr201, isr202, isr203, isr204, isr205, isr206, isr207,
            isr208, isr209, isr210, isr211, isr212, isr213, isr214, isr215,
            isr216, isr217, isr218, isr219, isr220, isr221, isr222, isr223,
            isr224, isr225, isr226, isr227, isr228, isr229, isr230, isr231,
            isr232, isr233, isr234, isr235, isr236, isr237, isr238, isr239,
            isr240, isr241, isr242, isr243, isr244, isr245, isr246, isr247,
            isr248, isr249, isr250, isr251, isr252, isr253, isr254, isr255,
        ];
        for (i, &addr) in handlers.iter().enumerate() {
            set_handler(i, addr as u32, 0x10, 0x8E);
        }

        init_pic(0x20, 0x28);

        IDTR.limit = (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16;
        IDTR.base = &IDT as *const _ as u32;

        setIdt(IDTR.limit, IDTR.base as *const u8);
    }
    kprint!(LogLevel::Info, "IDT initialized successfully\n");
}