global setGdt

global reloadSegments

section .data
gdtr	dw 0
		dd 0

section .text
setGdt:
    mov		AX, [esp + 4]
    mov		[gdtr], AX
    mov		EAX, [ESP + 8]
    mov		[gdtr + 2], EAX
    lgdt	[gdtr]
	ret

reloadSegments:
	jmp		0x10:.reload_CS

.reload_CS:
	mov		AX, 0x18
	mov		DS, AX
	mov		ES, AX
	mov		FS, AX
	mov		GS, AX
	mov		SS, AX
	ret