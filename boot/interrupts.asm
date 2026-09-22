bits 32

extern rust_interrupt_dispatch

section .text

; Exceptions for which the CPU does not push an error code. A zero is added so
; every handler presents the same stack layout to Rust.
%macro ISR_NO_ERROR 1
global isr%1
isr%1:
    push strict dword 0
    push strict dword %1
    jmp interrupt_common
%endmacro

; For these exceptions, the CPU has already pushed the error code.
%macro ISR_ERROR 1
global isr%1
isr%1:
    push strict dword %1
    jmp interrupt_common
%endmacro

ISR_NO_ERROR 0
ISR_NO_ERROR 1
ISR_NO_ERROR 2
ISR_NO_ERROR 3
ISR_NO_ERROR 4
ISR_NO_ERROR 5
ISR_NO_ERROR 6
ISR_NO_ERROR 7
ISR_ERROR    8
ISR_NO_ERROR 9
ISR_ERROR    10
ISR_ERROR    11
ISR_ERROR    12
ISR_ERROR    13
ISR_ERROR    14
ISR_NO_ERROR 15
ISR_NO_ERROR 16
ISR_ERROR    17
ISR_NO_ERROR 18
ISR_NO_ERROR 19
ISR_NO_ERROR 20
ISR_NO_ERROR 21
ISR_NO_ERROR 22
ISR_NO_ERROR 23
ISR_NO_ERROR 24
ISR_NO_ERROR 25
ISR_NO_ERROR 26
ISR_NO_ERROR 27
ISR_NO_ERROR 28
ISR_NO_ERROR 29
ISR_NO_ERROR 30
ISR_NO_ERROR 31

; PIC IRQs after remapping: IRQ0..15 become vectors 32..47.
%assign vector 32
%rep 16
ISR_NO_ERROR vector
%assign vector vector + 1
%endrep

; Kernel software interrupt used by the signal API (not a syscall gate).
ISR_NO_ERROR 48

; Safe fallback for every IDT entry which has no dedicated stub.
global isr_default
isr_default:
    push strict dword 0
    push strict dword 255
    jmp interrupt_common

interrupt_common:
    cld

    ; Save segment registers, then all general-purpose registers. pushad leaves
    ; the fields in the exact order described by InterruptFrame in Rust.
    push ds
    push es
    push fs
    push gs
    pushad

    ; Rust always executes with the kernel data selector active.
    mov ax, 0x10
    mov ds, ax
    mov es, ax

    push esp
    call rust_interrupt_dispatch
    add esp, 4

    popad
    pop gs
    pop fs
    pop es
    pop ds

    ; Drop vector + error code and restore EIP, CS and EFLAGS atomically.
    add esp, 8
    iretd

; Final stop path used after a panic or an explicit halt. The stack pointer is
; intentionally kept valid; every other general-purpose register is cleared.
global cpu_halt_clean
cpu_halt_clean:
    cli
    xor eax, eax
    xor ebx, ebx
    xor ecx, ecx
    xor edx, edx
    xor esi, esi
    xor edi, edi
    xor ebp, ebp
.halt:
    hlt
    jmp .halt

section .rodata
align 4
global interrupt_stub_table
interrupt_stub_table:
%assign vector 0
%rep 49
    dd isr %+ vector
%assign vector vector + 1
%endrep

section .note.GNU-stack noalloc noexec nowrite progbits
