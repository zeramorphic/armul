;! halts 200

b main

mem dw 0

main
        ; ldrh
        mov r0, 100
        str r0, mem
        ldrh r1, mem
        cmp r0, r1
        swine 100

        mov r0, 200
        str r0, mem
        ldrh r1, mem
        mov r0, 200
        cmp r0, r1
        swine 101

        mov r0, 0xABCDFEDC
        str r0, mem
        mov r1, 0xFFFFFFFF
        ldrh r1, mem
        mov r0, 0x0000FEDC
        cmp r0, r1
        swine 102

        ; ldrsb
        mov r0, 100
        str r0, mem
        ldrsb r1, mem
        cmp r0, r1
        swine 110

        mov r0, 200
        str r0, mem
        ldrsb r1, mem
        mov r0, 4294967240
        cmp r0, r1
        swine 111

        ; ldrsh
        mov r0, 100
        str r0, mem
        ldrsh r1, mem
        cmp r0, r1
        swine 120

        mov r0, 200
        str r0, mem
        ldrsh r1, mem
        mov r0, 200
        cmp r0, r1
        swine 121

        mov r0, 0x0000FEDC
        str r0, mem
        ldrsh r1, mem
        mov r0, 0xFFFFFEDC
        cmp r0, r1
        swine 122

        ; strh
        mov r0, 0
        str r0, mem
        mov r0, 0xABCD1234
        strh r0, mem
        mov r0, 0x1234
        ldr r1, mem
        cmp r0, r1
        swine 130

        swi 2
