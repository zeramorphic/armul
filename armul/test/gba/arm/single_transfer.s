; Tests for the single data transfer instruction

;! halts 200

FLAG_N equ 1 lsl 31
FLAG_Z equ 1 lsl 30
FLAG_C equ 1 lsl 29
FLAG_V equ 1 lsl 28

MEM_IWRAM equ 0x03000000
mov     r11, MEM_IWRAM

t350
        ; ARM 7: Load / store word
        mvn     r0, 0
        str     r0, [r11]
        ldr     r1, [r11]
        cmp     r1, r0
        bne     f350

        add     r11, 32
        b       t351

f350
        swi     350

t351
        ; ARM 7: Store byte
        mvn     r0, 0
        strb    r0, [r11]
        ldr     r1, [r11]
        cmp     r1, 0xFF
        bne     f351

        add     r11, 32
        b       t352

f351
        swi     351

t352
        ; ARM 7: Load byte
        mvn     r0, 0
        str     r0, [r11]
        ldrb    r1, [r11]
        cmp     r1, 0xFF
        bne     f352

        add     r11, 32
        b       t353

f352
        swi     352

t353
        ; ARM 7: Indexing, writeback and offset types
        mov     r0, 32
        mov     r1, 1
        mov     r2, r11
        str     r0, [r2], 4
        ldr     r3, [r2, -r1, lsl 2]!
        cmp     r3, r0
        bne     f353
        cmp     r2, r11
        bne     f353

        add     r11, 32
        b       t354

f353
        swi     353

t354
        ; ARM 7: Misaligned store
        mov     r0, 32
        str     r0, [r11, 3]
        ldr     r1, [r11]
        cmp     r1, r0
        bne     f354

        add     r11, 32
        b       t355

f354
        swi     354

t355
        ; ARM 7: Misaligned load (rotated)
        mov     r0, 32
        str     r0, [r11]
        ldr     r1, [r11, 3]
        cmp     r1, r0, ror 24
        bne     f355

        add     r11, 32
        b       t356

f355
        swi     355

t356
        ; ARM 7: Store PC + 4
        str     pc, [r11]
        mov     r0, pc
        ldr     r1, [r11]
        cmp     r1, r0
        bne     f356

        add     r11, 32
        b       t357

f356
        swi     356

t357
        ; ARM 7: Load into PC
        adr     r0, t358
        str     r0, [r11]
        ldr     pc, [r11], 32

f357
        swi     357

t358
        ; ARM 7: Store writeback same register
        mov     r0, r11
        dw      0xE5A00004  ; str r0, [r0, 4]!
        add     r1, r11, 4
        cmp     r1, r0
        bne     f358

        ldr     r1, [r0]
        cmp     r1, r11
        bne     f358

        add     r11, 32
        b       t359

f358
        swi     358

t359
        ; ARM 7: Store writeback same register
        mov     r0, r11
        dw      0xE4800004  ; str r0, [r0], 4
        sub     r0, 4
        cmp     r0, r11
        bne     f359

        ldr     r1, [r0]
        cmp     r1, r11
        bne     f359

        add     r11, 32
        b       t360

f359
        swi     359

t360
        ; ARM 7: Load writeback same register
        mov     r0, r11
        mov     r1, 32
        str     r1, [r0], -4
        dw      0xE5B00004  ; ldr r0, [r0, 4]!
        cmp     r0, 32
        bne     f360

        add     r11, 32
        b       t361

f360
        swi     360

t361
        ; ARM 7: Load writeback same register
        mov     r0, r11
        mov     r1, 32
        str     r1, [r0]
        dw      0xE4900004  ; ldr r0, [r0], 4
        cmp     r0, 32
        bne     f361

        add     r11, 32
        b       t362

f361
        swi     361

t362
        ; ARM 7: Special shifts as offset
        mov     r0, 0
        mov     r1, 0
        msr     cpsr_flg, FLAG_C
        ldr     r2, [r1, r0, rrx]!
        cmp     r1, 1 lsl 31
        bne     f362
        bcc     f362

        add     r11, 32
        b       t363

f362
        swi     362

t363
        ; ARM 7: Load current instruction
        ldr     r0, [pc, -8]
        mov     r1, 0xE51F0008
        bne     f363

        add     r11, 32
        b       single_transfer_passed

f363
        swi     363

single_transfer_passed
