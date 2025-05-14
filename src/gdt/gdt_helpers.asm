[BITS 32]
global load_gdt
global gdt_reload_segments

section .data
; Data structure for far jump target
reload_target:
    dd reload_cs        ; 32-bit offset for far jump
    dw 0x0              ; Selector will be filled at runtime

section .text
; Function to load the GDT register
; void load_gdt(void* gdtr_ptr);
load_gdt:
    mov eax, [esp+4]    ; Get pointer to GDTR
    lgdt [eax]          ; Load GDTR
    ret

; Function to reload segment registers
; void gdt_reload_segments(uint16_t code_selector, uint16_t data_selector);
gdt_reload_segments:
    ; Get arguments
    mov ax, [esp+4]     ; Code selector
    mov bx, [esp+8]     ; Data selector
    
    ; Store the code selector in our far jump structure
    mov [reload_target+4], ax
    
    ; First reload the data segment registers
    mov ds, bx
    mov es, bx
    mov fs, bx
    mov gs, bx
    ; Don't reload SS yet, as that could cause stack issues
    
    ; Perform a far jump to reload CS
    jmp 0x08:reload_cs  ; Direct far jump with immediate segment and offset
    
reload_cs:
    ; Now that CS is reloaded, we can safely set SS
    mov ss, bx
    
    ret