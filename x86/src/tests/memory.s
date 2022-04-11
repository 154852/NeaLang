; Build with
; nasm -f bin x86/src/tests/memory.s -o x86/src/tests/memory.bin

[BITS 64]

global memory
memory:
    mov rax, [123]
    mov rax, [rcx + 2]
    mov rax, [rcx + rdx]
    mov rax, [rcx + 4*rdx]
    mov rax, [rcx + 4*rdx + 10]