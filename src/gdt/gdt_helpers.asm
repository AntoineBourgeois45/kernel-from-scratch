global load_gdt
global gdt_reload_segments

section .text
load_gdt:
    mov eax, [esp+4]    ; Get pointer to GDTR
    lgdt [eax]          ; Load GDTR
    ret

gdt_reload_segments:
    ; Utiliser directement les sélecteurs connus
    mov ax, 0x10        ; Code segment (indice 2)
    mov bx, 0x18        ; Data segment (indice 3)
    
    ; Recharger les registres de données
    mov ds, bx
    mov es, bx
    mov fs, bx
    mov gs, bx
    
    ; Recharger CS via un saut lointain
    ; Utiliser un saut direct plutôt qu'indirect
    jmp 0x10:reload_cs
    
reload_cs:
    ; Maintenant recharger SS
    mov ss, bx
    ret