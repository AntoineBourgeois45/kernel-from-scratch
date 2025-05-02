ENTRY(_start)

SECTIONS
{
    /* Place le point d’entrée à 1 MiB */
    . = 1M;

    .multiboot : {
        *(.multiboot)
    }

    .text : {
        *(.text)
    }

    /* Aligne l’adresse courante sur 4 KiB avant rodata */
    . = ALIGN(0x1000);
    .rodata : ALIGN(0x1000) {
        *(.rodata)
    }

    /* Même principe pour .data */
    . = ALIGN(0x1000);
    .data : ALIGN(0x1000) {
        *(.data)
    }

    /* Et pour .bss (COMMON + bss) */
    . = ALIGN(0x1000);
    .bss : ALIGN(0x1000) {
        *(COMMON)
        *(.bss)
    }
}
