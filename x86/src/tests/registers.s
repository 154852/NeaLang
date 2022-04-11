; Build with
; nasm -f bin x86/src/tests/registers.s -o x86/src/tests/registers.bin

[BITS 64]

global registers
registers:
    mov rax, rcx
    mov rax, rdx
    mov rax, rbx
    mov rax, rdi
    mov rax, rsi
    mov rax, rbp
    mov rax, rsp
    mov rax, r8
    mov rax, r9
    mov rax, r10
    mov rax, r11
    mov rax, r12
    mov rax, r13
    mov rax, r14
    mov rax, r15

    mov r8, rax
    mov r9, rax
    mov r10, rax
    mov r11, rax
    mov r12, rax
    mov r13, rax
    mov r14, rax
    mov r15, rax

    mov r8, r8
    mov r8, r9
    mov r8, r10
    mov r8, r11
    mov r8, r12
    mov r8, r13
    mov r8, r14
    mov r8, r15

    mov rax, [r8]
    mov r8, [rax]
    mov [r8], rax
    mov [rax], r8

    mov rax, [r8+rbx]
    mov rax, [rbx+r8]
    mov rax, [r9+r8]
    mov r10, [r9+r8]