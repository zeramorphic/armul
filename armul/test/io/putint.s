;! halts 30
;! output 10 16 -30

    mov r0, 10
    swi 4
    mov r0, 0x20 ; ' '
    swi 0
    mov r1, 16
    bl putint
    mov r0, 0x20 ; ' '
    swi 0
    mov r1, -30
    bl putint
    swi 2

putint
    cmp r1, 0
    movlt r0, 0x2D ; '-'
    swilt 0
    mvnlt r0, r1
    addlt r0, r0, 1
    movge r0, r1
    swi 4
    mov pc, lr
