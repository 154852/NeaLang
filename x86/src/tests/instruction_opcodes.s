; Build with
; nasm -f bin x86/src/tests/instruction_opcodes.s -o x86/src/tests/instruction_opcodes.bin

[BITS 64]

global instruction_opcodes
instruction_opcodes:
    add rax, rcx ; AddRegReg
    add [rdi], rcx ; AddMemReg
    
    and rax, rcx ; AndRegReg
    and [rdi], rcx ; AndMemReg

    cwd ; Cdq(Size::Quad)

    cmove rax, rcx ; CMovRegReg
    cmovl rax, [rdi] ; CMovRegMem

    cmp rax, rcx ; CmpRegReg
    cmp rax, [rdi] ; CmpRegMem

    setge al ; ConditionalSet

    div rcx ; DivReg
    div qword [rdi] ; DivMem

    idiv rcx ; IDivReg
    idiv qword [rdi] ; IDivMem

    imul rax, rcx ; IMulRegReg
    imul rax, [rdi] ; IMulRegMem

    lea rax, [rdi] ; LeaRegMem

    mov rax, rcx ; MovRegReg
    mov rax, [rdi] ; MovRegMem
    mov [rax], rdi ; MovMemReg
    mov rax, 42 ; MovRegImm
    mov qword [rdi], 42 ; MovMemImm

    movsx rax, ecx ; MovsxRegReg
    movsx rax, dword [rdi] ; MovsxRegMem

    movzx eax, cx ; MovzxRegReg
    movzx eax, word [rdi] ; MovzxRegMem

    neg rax ; NegReg
    neg qword [rax]

    or rax, rcx ; OrRegReg
    or rax, [rdi] ; OrRegMem

    pop rax ; PopReg
    pop qword [rdi] ; PopMem

    push rax ; PushReg
    push qword [rdi] ; PushMem
    push 32 ; PushImm

    ret ; Return

    sub rax, rcx ; SubRegReg
    sub rax, [rdi] ; SubRegMem

    test rax, rcx ; TestRegReg
    test [rdi], rax ; TestMemReg

    add rax, 3
    add rax, 356
    add rcx, 7
    add rcx, 456
    add qword [rcx], 34