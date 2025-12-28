; Tests for branch operations

;! halts 50
;! r1 8

        mov r1, #0

t050
        ; ARM 1: Branch with exchange
        mov     r12, #50
        adr     r0, t052
        add     r1, r1, #1
        bx      r0

t052
        ; ARM 1: Branch without exchange
        mov     r12, #52
        adr     r0, t053
        add     r1, r1, #1
        bx      r0

t053
        ; ARM 2: Branch forward
        mov     r12, #53
        add     r1, r1, #1
        b       t054

t055
        ; ARM 2: Branch forward
        mov     r12, #55
        add     r1, r1, #1
        b       t056

t054
        ; ARM 2: Branch backward
        mov     r12, #54
        add     r1, r1, #1
        b       t055

t057
        ; ARM 2: Test link
        mov     r12, #57
        add     r1, r1, #1
        mov     pc, lr

t056
        ; ARM 2: Branch with link
        mov     r12, #56
        add     r1, r1, #1
        bl      t057

branches_passed
        mov     r12, 0
        add     r1, r1, #1
        swi     2
