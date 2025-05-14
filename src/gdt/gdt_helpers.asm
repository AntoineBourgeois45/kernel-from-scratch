[BITS 32]
global gdt_reload_segments

section .text
gdt_reload_segments:
    ; Lire les paramètres directement
    movzx eax, word [esp+4]  ; Sélecteur de code (16 bits avec zero extension)
    movzx ebx, word [esp+8]  ; Sélecteur de données (16 bits avec zero extension)
    
    ; Sauvegarder le sélecteur de données dans un registre préservé
    mov edi, ebx
    
    ; Préparer l'adresse de retour pour l'étiquette
    mov edx, reload_cs_done
    
    ; Configuration pour le far return qui changera CS
    push ax                  ; Pousser le sélecteur de code (16 bits)
    push edx                 ; Pousser l'adresse de retour (32 bits)
    
    ; Exécuter le far return
    retf
    
reload_cs_done:
    ; CS est maintenant rechargé avec le nouveau sélecteur
    
    ; Recharger les segments de données
    mov ax, di               ; Récupérer le sélecteur de données
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax               ; Charger SS en dernier
    
    ; Retourner à l'appelant
    ret