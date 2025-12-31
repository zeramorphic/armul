; Tests for the block transfer instruction

;! halts 200

MODE_USR equ 0x10
MODE_FIQ equ 0x11
MODE_IRQ equ 0x12
MODE_SVC equ 0x13
MODE_ABT equ 0x17
MODE_SYS equ 0x1F

FLAG_N equ 1 lsl 31
FLAG_Z equ 1 lsl 30
FLAG_C equ 1 lsl 29
FLAG_V equ 1 lsl 28

MEM_IWRAM equ 0x03000000
mov     r11, MEM_IWRAM

t500
        ; ARM 10: Fully ascending
        mov     r0, 32
        mov     r1, 64
        stmfa   r11!, {r0, r1}
        ldmfa   r11!, {r2, r3}
        cmp     r0, r2
        bne     f500
        cmp     r1, r3
        bne     f500

        add     r11, 32
        b       t501

f500
        swi     500

t501
        ; ARM 10: Empty ascending
        mov     r0, 32
        mov     r1, 64
        stmea   r11!, {r0, r1}
        ldmea   r11!, {r2, r3}
        cmp     r0, r2
        bne     f501
        cmp     r1, r3
        bne     f501

        add     r11, 32
        b       t502

f501
        swi     501

t502
        ; ARM 10: Fully descending
        mov     r0, 32
        mov     r1, 64
        stmfd   r11!, {r0, r1}
        ldmfd   r11!, {r2, r3}
        cmp     r0, r2
        bne     f502
        cmp     r1, r3
        bne     f502

        add     r11, 32
        b       t503

f502
        swi     502

t503
        ; ARM 10: Empty descending
        mov     r0, 32
        mov     r1, 64
        stmed   r11!, {r0, r1}
        ldmed   r11!, {r2, r3}
        cmp     r0, r2
        bne     f503
        cmp     r1, r3
        bne     f503

        add     r11, 32
        b       t504

f503
        swi     503

t504
        ; ARM 10: Location fully ascending
        mov     r0, 32
        stmfa   r11, {r0, r1}
        ldr     r1, [r11, 4]
        cmp     r1, r0
        bne     f504

        add     r11, 32
        b       t505

f504
        swi     504

t505
        ; ARM 10: Location empty ascending
        mov     r0, 32
        stmea   r11, {r0, r1}
        ldr     r1, [r11]
        cmp     r1, r0
        bne     f505

        add     r11, 32
        b       t506

f505
        swi     505

t506
        ; ARM 10: Location fully descending
        mov     r0, 32
        stmfd   r11, {r0, r1}
        ldr     r1, [r11, -8]
        cmp     r1, r0
        bne     f506

        add     r11, 32
        b       t507

f506
        swi     506

t507
        ; ARM 10: Location empty descending
        mov     r0, 32
        stmed   r11, {r0, r1}
        ldr     r1, [r11, -4]
        cmp     r1, r0
        bne     f507

        add     r11, 32
        b       t508

f507
        swi     507

t508
        ; ARM 10: Memory alignment
        mov     r0, 32
        mov     r1, 64
        add     r2, r11, 3
        sub     r3, r11, 5
        stmfd   r2!, {r0, r1}
        ldmfd   r3, {r4, r5}
        cmp     r0, r4
        bne     f508
        cmp     r1, r5
        bne     f508
        cmp     r2, r3
        bne     f508

        add     r11, 32
        b       t509

f508
        swi     508

t509
        ; ARM 10: Load PC
        adr     r1, t510
        stmfd   r11!, {r0, r1}
        ldmfd   r11!, {r0, pc}

f509
        swi     509

t510
        ; ARM 10: Store PC + 4
        stmfd   r11!, {r0, pc}
        mov     r0, pc
        ldmfd   r11!, {r1, r2}
        cmp     r0, r2
        bne     f510

        add     r11, 32
        b       t511

f510
        swi     510

t511
        ; ARM 10: Store user registers
        mov     r0, r11
        mov     r8, 32
        mov     r2, MODE_FIQ
        msr     cpsr, r2
        mov     r8, 64
        stmfd   r0, {r8, r9}^
        sub     r0, 8
        mov     r2, MODE_SYS
        msr     cpsr, r2
        ldmfd   r0, {r1, r2}
        cmp     r1, 32
        bne     f511

        add     r11, 32
        b       t512

f511
        swi     511

t512
        ; ARM 10: Load user registers
        mov     r0, r11
        mov     r1, 0xA
        stmfd   r0!, {r1, r2}
        mov     r2, MODE_FIQ
        msr     cpsr, r2
        mov     r8, 0xB
        ldmfd   r0, {r8, r9}^
        cmp     r8, 0xB
        bne     f512
        mov     r2, MODE_SYS
        msr     cpsr, r2
        cmp     r8, 0xA
        bne     f512

        add     r11, 32
        b       t513

f512
        swi     512

t513
        ; ARM 10: Load empty rlist
        adr     r0, t514
        str     r0, [r11]
        mov     r0, r11
        dw      0xE8B00000  ; ldmia r0!, {}

f513
        swi     513

t514
        sub     r0, 0x40
        cmp     r0, r11
        bne     f514

        add     r11, 32
        b       t515

f514
        swi     514

t515
        ; ARM 10: STMIA empty rlist
        mov     r0, r11
        dw      0xE8A00000  ; stmia r0!, {}
        mov     r1, pc
        ldr     r2, [r11]
        cmp     r2, r1
        bne     f515

        sub     r0, 0x40
        cmp     r0, r11
        bne     f515

        add     r11, 32
        b       t516

f515
        swi     515

t516
        ; ARM 10: Load writeback base first in rlist
        mov     r0, 0xA
        mov     r1, r11
        stmfd   r1!, {r0, r2}
        dw      0xE8B10006  ; ldmfa r1!, {r1, r2}
        cmp     r1, 0xA
        bne     f516

        add     r11, 32
        b       t517

f516
        swi     516

t517
        ; ARM 10: Load writeback base last in rlist
        mov     r2, 0xA
        mov     r1, r11
        stmfd   r1!, {r0, r2}
        dw      0xE8B10003  ; ldmfa r1!, {r0, r1}
        cmp     r1, 0xA
        bne     f517

        add     r11, 32
        b       t518

f517
        swi     517

t518
        ; ARM 10: STMFD base first in rlist
        mov     r0, r11
        stmfd   r0!, {r0, r1}
        ldmfd   r0!, {r1, r2}
        cmp     r1, r11
        bne     f518

        add     r11, 32
        b       t519

f518
        swi     518

t519
        ; ARM 10: STMED base first in rlist
        mov     r0, r11
        stmed   r0!, {r0, r1}
        ldmed   r0!, {r1, r2}
        cmp     r1, r11
        bne     f519

        add     r11, 32
        b       t520

f519
        swi     519

t520
        ; ARM 10: STMFA base first in rlist
        mov     r0, r11
        stmfa   r0!, {r0, r1}
        ldmfa   r0!, {r1, r2}
        cmp     r1, r11
        bne     f520

        add     r11, 32
        b       t521

f520
        swi     520

t521
        ; ARM 10: STMEA base first in rlist
        mov     r0, r11
        stmea   r0!, {r0, r1}
        ldmea   r0!, {r1, r2}
        cmp     r1, r11
        bne     f521

        add     r11, 32
        b       t522

f521
        swi     521

t522
        ; ARM 10: STMFD base in rlist
        mov     r1, r11
        dw      0xE921000F  ; stmfd r1!, {r0-r3}
        ldmfd   r1!, {r4-r7}
        sub     r0, r11, 16
        cmp     r0, r5
        bne     f522

        add     r11, 32
        b       t523

f522
        swi     522

t523
        ; ARM 10: STMFD base in rlist
        mov     r2, r11
        dw      0xE922000F  ; stmfd r2!, {r0-r3}
        ldmfd   r2!, {r4-r7}
        sub     r0, r11, 16
        cmp     r0, r6
        bne     f523

        add     r11, 32
        b       t524

f523
        swi     523

t524
        ; ARM 10: STMED base in rlist
        mov     r1, r11
        dw      0xE821000F  ; stmed r1!, {r0-r3}
        ldmed   r1!, {r4-r7}
        sub     r0, r11, 16
        cmp     r0, r5
        bne     f524

        add     r11, 32
        b       t525

f524
        swi     524

t525
        ; ARM 10: STMED base in rlist
        mov     r2, r11
        dw      0xE822000F  ; stmed r2!, {r0-r3}
        ldmed   r2!, {r4-r7}
        sub     r0, r11, 16
        cmp     r0, r6
        bne     f525

        add     r11, 32
        b       t526

f525
        swi     525

t526
        ; ARM 10: STMFA base in rlist
        mov     r1, r11
        dw      0xE9A1000F  ; stmfa r1!, {r0-r3}
        ldmfa   r1!, {r4-r7}
        add     r0, r11, 16
        cmp     r0, r5
        bne     f526

        add     r11, 32
        b       t527

f526
        swi     526

t527
        ; ARM 10: STMFA base in rlist
        mov     r2, r11
        dw      0xE9A2000F  ; stmfa r2!, {r0-r3}
        ldmfa   r2!, {r4-r7}
        add     r0, r11, 16
        cmp     r0, r6
        bne     f527

        add     r11, 32
        b       t528

f527
        swi     527

t528
        ; ARM 10: STMEA base in rlist
        mov     r1, r11
        dw      0xE8A1000F  ; stmea r1!, {r0-r3}
        ldmea   r1!, {r4-r7}
        add     r0, r11, 16
        cmp     r0, r5
        bne     f528

        add     r11, 32
        b       t529

f528
        swi     528

t529
        ; ARM 10: STMEA base in rlist
        mov     r2, r11
        dw      0xE8A2000F  ; stmea r2!, {r0-r3}
        ldmea   r2!, {r4-r7}
        add     r0, r11, 16
        cmp     r0, r6
        bne     f529

        add     r11, 32
        b       t530

f529
        swi     529

t530
        ; ARM 10: STMDA empty rlist
        mov     r0, r11
        mov     r1, 0
        str     r1, [r11, -60]
        dw      0xE8200000  ; stmda r0!, {}
        mov     r1, pc
        ldr     r2, [r11, -60]
        cmp     r2, r1
        bne     f530

        add     r0, 0x40
        cmp     r0, r11
        bne     f530

        add     r11, 32
        b       t531

f530
        swi     530

t531
        ; ARM 10: STMDB empty rlist
        mov     r0, r11
        mov     r1, 0
        str     r1, [r11, -64]
        dw      0xE9200000  ; stmdb r0!, {}
        mov     r1, pc
        ldr     r2, [r11, -64]
        cmp     r2, r1
        bne     f531

        add     r0, 64
        cmp     r0, r11
        bne     f531

        add     r11, 32
        b       t532

f531
        swi     531

t532
        ; ARM 10: STMIB empty rlist
        mov     r0, r11
        mov     r1, 0
        str     r1, [r11, 4]
        dw      0xE9A00000  ; stmib r0!, {}
        mov     r1, pc
        ldr     r2, [r11, 4]
        cmp     r2, r1
        bne     f532

        sub     r0, 0x40
        cmp     r0, r11
        bne     f532

        add     r11, 32
        b       block_transfer_passed

f532
        swi     532

block_transfer_passed
        swi     2
