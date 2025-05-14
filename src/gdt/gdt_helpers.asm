[BITS 32]
global gdt_reload_segments

section .text
gdt_reload_segments:
    ; Récupération des arguments
    mov eax, [esp+4]  ; code selector
    mov ebx, [esp+8]  ; data selector
    
    ; Far jump pour recharger CS
    ; Structure: jmp FAR [mem] où mem contient [offset, segment]
    jmp far [reload_target]

reload_target:
    dd reload_segments  ; offset 32 bits
    dw 0                ; segment selector (rempli dynamiquement)
    
reload_segments:
    ; À ce stade, CS est rechargé
    ; Remplir le sélecteur de segment avec la valeur dans eax
    mov [reload_target+4], ax
    
    ; Recharger les segments de données
    mov ds, bx
    mov es, bx
    mov fs, bx
    mov gs, bx
    mov ss, bx
    
    ret

section .data
    ; Espace pour l'alignement
    dd 0