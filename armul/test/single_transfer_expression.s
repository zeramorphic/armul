;! halts 10

;! r0 100
;! r1 200

b main

behind
        dw 200

main
        ldr     r0, ahead
        ldr     r1, behind
        swi     2

ahead
        dw 100
