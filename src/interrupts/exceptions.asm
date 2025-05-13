BITS 32

section .data
    global interrupt_number
    interrupt_number:    db 0

section .text
    extern exception_handler
    global int_bottom

int_bottom:
    pusha
    push    ds
    push    es
    push    fs
    push    gs

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call    exception_handler


    pop     gs
    pop     fs
    pop     es
    pop     ds
    popa
    
    add     esp, 8
    iret

%macro ISR_STUB 1
global isr%1
isr%1:
    mov byte [interrupt_number], %1
    jmp int_bottom
%endmacro

ISR_STUB 0
ISR_STUB 1
ISR_STUB 2
ISR_STUB 3
ISR_STUB 4
ISR_STUB 5
ISR_STUB 6
ISR_STUB 7
ISR_STUB 8
ISR_STUB 9
ISR_STUB 10
ISR_STUB 11
ISR_STUB 12
ISR_STUB 13
ISR_STUB 14
ISR_STUB 15
ISR_STUB 16
ISR_STUB 17
ISR_STUB 18
ISR_STUB 19
ISR_STUB 20
ISR_STUB 21
ISR_STUB 22
ISR_STUB 23
ISR_STUB 24
ISR_STUB 25
ISR_STUB 26
ISR_STUB 27
ISR_STUB 28
ISR_STUB 29
ISR_STUB 30
ISR_STUB 31
