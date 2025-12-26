        mov r1,#37
        mov r2,#6
        mov r0,#1
div1    cmp r2,#0x80100101
        cmpcc r2,r1
        movcc r2,r2,asl#1
        movcc r0,r0,asl#1
        bcc div1
        mov r3,#0
div2    cmp r1,r2
        subcs r1,r1,r2
        addcs r3,r3,r0
        movs r0,r0,lsr#1
        movne r2,r2,lsr#1
        bne div2
