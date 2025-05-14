ENTRY(_start)

SECTIONS
{
    . = 1M;

    .multiboot : {
        *(.multiboot)
    }

    .text : {
        *(.text)
    }

    . = ALIGN(0x1000);
    .rodata : ALIGN(0x1000) {
        *(.rodata)
    }

    . = ALIGN(0x1000);
    .data : ALIGN(0x1000) {
        *(.data)
    }

    . = ALIGN(0x1000);
    .bss : ALIGN(0x1000) {
        *(COMMON)
        *(.bss)
    }
}
