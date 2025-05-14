[BITS 32]
global gdt_reload_segments

section .text
gdt_reload_segments:
    ; Get arguments from stack (cdecl calling convention)
    mov ax, [esp+4]  ; code selector
    mov bx, [esp+8]  ; data selector
    
    ; Create a far jump structure
    mov [temp_gdt.code_segment], ax  ; Store code segment selector
    jmp far [temp_gdt]               ; Far jump using the structure
    
temp_gdt:
    dd reload_segments      ; 32-bit offset (where to jump to)
.code_segment:
    dw 0                    ; 16-bit segment selector (filled at runtime)
    
reload_segments:
    ; Now we're executing with the new code segment
    ; Reload all data segment registers with new data segment
    mov ds, bx
    mov es, bx
    mov fs, bx
    mov gs, bx
    mov ss, bx
    
    ret