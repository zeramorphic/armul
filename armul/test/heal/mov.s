; Test MOV healing.

;! halts 50
;! r0 -1
;! r1 -100
;! r2 313221886
;! r3 -1610567680

        mov r0, #-1
        mov r1, #-100
        mov r2, #0x12AB62FE
        mov r3, 0xA000B000
        swi 2
