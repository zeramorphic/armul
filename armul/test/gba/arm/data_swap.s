;! halts 100

FLAG_N equ 1 lsl 31
FLAG_Z equ 1 lsl 30
FLAG_C equ 1 lsl 29
FLAG_V equ 1 lsl 28

MEM_IWRAM equ 0x03000000
mov     r11, MEM_IWRAM

t450
        ; ARM 10 Swap word
        mvn     r0, 0
        str     r0, [r11]
        swp     r1, r0, [r11]
        cmp     r1, r0
        bne     f450
        ldr     r1, [r11]
        cmp     r1, r0
        bne     f450

        add     r11, 32
        b       t451

f450
        swi     450

t451
        ; ARM 10 Swap byte
        mvn     r0, 0
        str     r0, [r11]
        swpb    r1, r0, [r11]
        cmp     r1, 0xFF
        bne     f451
        ldr     r1, [r11]
        cmp     r1, r0
        bne     f451

        add     r11, 32
        b       t452

f451
        swi     451

t452
        ; ARM 10 Misaligned swap
        mov     r0, 32
        mov     r1, 64
        str     r1, [r11]
        add     r2, r11, 1
        swp     r3, r0, [r2]
        cmp     r3, r1, ror 8
        bne     f452
        ldr     r3, [r11]
        cmp     r3, r0
        bne     f452

        add     r11, 32
        b       t453

f452
        swi     452

t453
        ; ARM 10 Same source and destination
        mov     r0, 32
        str     r0, [r11]
        mov     r0, 64
        swp     r0, r0, [r11]
        cmp     r0, 32
        bne     f453
        ldr     r0, [r11]
        cmp     r0, 64
        bne     f453

        b       data_swap_passed

f453
        swi     453

data_swap_passed
        swi     2
