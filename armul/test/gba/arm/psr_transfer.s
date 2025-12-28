; Tests for the PSR transfer instruction

;! halts 100

MODE_USR equ 0x10
MODE_FIQ equ 0x11
MODE_IRQ equ 0x12
MODE_SVC equ 0x13
MODE_ABT equ 0x17
MODE_SYS equ 0x1F

t250
        ; ARM 4 Read / write PSR
        mrs     r0, cpsr
        bic     r0, r0, 0xF0000000
        msr     cpsr, r0
        beq     f250
        bmi     f250
        bcs     f250
        bvs     f250

        b       t251

f250
        swi     250

t251
        ; ARM 4 Write flag bits
        msr     cpsr_flg, 0xF0000000
        bne     f251
        bpl     f251
        bcc     f251
        bvc     f251

        b       t252

f251
        swi     251

t252
        ; ARM 4 Write control bits
        mov     r7, MODE_FIQ
        msr     cpsr, r7
        mrs     r0, cpsr
        and     r0, r0, 0x1F
        cmp     r0, MODE_FIQ
        bne     f252

        mov     r7, MODE_SYS
        msr     cpsr, r7

        b       t253

f252
        swi     252

t253
        ; ARM 4 Register banking
        mov     r0, 16
        mov     r8, 32
        mov     r7, MODE_FIQ
        msr     cpsr, r7
        mov     r0, 32
        mov     r8, 64
        mov     r7, MODE_SYS
        msr     cpsr, r7
        cmp     r0, 32
        bne     f253
        cmp     r8, 32
        bne     f253

        b       t254

f253
        swi     253

t254
        ; ARM 4 Accessing SPSR
        mov     r7, MODE_FIQ
        msr     cpsr, r7
        mrs     r0, cpsr
        msr     spsr, r0
        mrs     r1, spsr
        mov     r7, MODE_SYS
        msr     cpsr, r7
        cmp     r1, r0
        bne     f254

        b       psr_transfer_passed

f254
        swi     254

psr_transfer_passed
        swi     2
