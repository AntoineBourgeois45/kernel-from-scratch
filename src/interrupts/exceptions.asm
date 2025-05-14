BITS 32

section .data
global interrupt_number
interrupt_number: db 0

section .text
extern exception_handler
extern handle_interrupt

; Common handler for CPU exceptions
%macro ISR_COMMON 0
    ; Save all registers
    pusha
    push ds
    push es
    push fs
    push gs
    
    ; Load kernel data segments
    mov ax, 0x18  ; Kernel data segment selector
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; Call the C handler
    call exception_handler
    
    ; We'll never return here for exceptions that use the ! return type,
    ; but in case we handle some exceptions differently in the future:
    pop gs
    pop fs
    pop es
    pop ds
    popa
    add esp, 8  ; Clean up error code and interrupt number
    iret
%endmacro

; Common handler for hardware interrupts (IRQs)
%macro IRQ_COMMON 0
    ; Save all registers
    pusha
    push ds
    push es
    push fs
    push gs
    
    ; Load kernel data segments
    mov ax, 0x18  ; Kernel data segment selector
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; Call the C handler with interrupt number as parameter
    movzx eax, byte [interrupt_number]
    push eax
    call handle_interrupt
    add esp, 4  ; Clean up parameter
    
    ; Restore registers
    pop gs
    pop fs
    pop es
    pop ds
    popa
    add esp, 8  ; Clean up error code and interrupt number
    iret
%endmacro

; ISR handlers for CPU exceptions
%macro ISR_NOERRCODE 1
global isr%1
isr%1:
    push dword 0  ; Push dummy error code
    push dword %1 ; Push interrupt number
    mov byte [interrupt_number], %1
    jmp isr_common
%endmacro

%macro ISR_ERRCODE 1
global isr%1
isr%1:
    ; Error code already pushed by CPU
    push dword %1 ; Push interrupt number
    mov byte [interrupt_number], %1
    jmp isr_common
%endmacro

; IRQ handlers for hardware interrupts
%macro IRQ 2
global isr%1
isr%1:
    push dword 0  ; Push dummy error code
    push dword %1 ; Push interrupt number
    mov byte [interrupt_number], %1
    jmp irq_common
%endmacro

; Exception handlers
ISR_NOERRCODE 0  ; Divide by zero
ISR_NOERRCODE 1  ; Debug
ISR_NOERRCODE 2  ; Non-maskable interrupt
ISR_NOERRCODE 3  ; Breakpoint
ISR_NOERRCODE 4  ; Overflow
ISR_NOERRCODE 5  ; Bound range exceeded
ISR_NOERRCODE 6  ; Invalid opcode
ISR_NOERRCODE 7  ; Device not available
ISR_ERRCODE   8  ; Double fault
ISR_NOERRCODE 9  ; Coprocessor segment overrun
ISR_ERRCODE   10 ; Invalid TSS
ISR_ERRCODE   11 ; Segment not present
ISR_ERRCODE   12 ; Stack-segment fault
ISR_ERRCODE   13 ; General protection fault
ISR_ERRCODE   14 ; Page fault
ISR_NOERRCODE 15 ; Reserved
ISR_NOERRCODE 16 ; x87 FPU error
ISR_ERRCODE   17 ; Alignment check
ISR_NOERRCODE 18 ; Machine check
ISR_NOERRCODE 19 ; SIMD floating-point exception
ISR_NOERRCODE 20 ; Virtualization exception
ISR_NOERRCODE 21 ; Reserved
ISR_NOERRCODE 22 ; Reserved
ISR_NOERRCODE 23 ; Reserved
ISR_NOERRCODE 24 ; Reserved
ISR_NOERRCODE 25 ; Reserved
ISR_NOERRCODE 26 ; Reserved
ISR_NOERRCODE 27 ; Reserved
ISR_NOERRCODE 28 ; Reserved
ISR_NOERRCODE 29 ; Reserved
ISR_NOERRCODE 30 ; Reserved
ISR_NOERRCODE 31 ; Reserved

; IRQ handlers (hardware interrupts)
IRQ 32, 0  ; IRQ0: Timer
IRQ 33, 1  ; IRQ1: Keyboard
IRQ 34, 2  ; IRQ2: Cascade for 8259A Slave controller
IRQ 35, 3  ; IRQ3: COM2
IRQ 36, 4  ; IRQ4: COM1
IRQ 37, 5  ; IRQ5: LPT2
IRQ 38, 6  ; IRQ6: Floppy disk
IRQ 39, 7  ; IRQ7: LPT1
IRQ 40, 8  ; IRQ8: CMOS Real-time clock
IRQ 41, 9  ; IRQ9: Free / SCSI / NIC
IRQ 42, 10 ; IRQ10: Free / SCSI / NIC
IRQ 43, 11 ; IRQ11: Free / SCSI / NIC
IRQ 44, 12 ; IRQ12: PS/2 Mouse
IRQ 45, 13 ; IRQ13: FPU / Coprocessor / Inter-processor
IRQ 46, 14 ; IRQ14: Primary ATA Hard Disk
IRQ 47, 15 ; IRQ15: Secondary ATA Hard Disk

; Common handlers
isr_common:
    ISR_COMMON

irq_common:
    IRQ_COMMON