; Test healing for non-MOV instructions.
; Uses R12 for the temp value.

;! halts 7
;! r0 102938476

    mov r0, #1
    add r0, r0, #102938475
    swi 2
