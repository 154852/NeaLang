; Build with
; nasm -f bin x86/src/tests/relocations.s -o x86/src/tests/relocations.bin

[BITS 64]

global relocations
relocations:
    jmp .a
    add rax, 3
.a:
    add rcx, 4
    jne .a