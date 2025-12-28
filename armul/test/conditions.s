; Conditions test suite adapted from <https://github.com/jsmolka/gba-tests>,
; licensed under the MIT License.

;! halts 500

FLAG_N equ 1 lsl 31
FLAG_Z equ 1 lsl 30
FLAG_C equ 1 lsl 29
FLAG_V equ 1 lsl 28

t001
        ; EQ - Z set
        msr     cpsr_flg, FLAG_Z
        beq     t002
        mov     r0, 1
        b       fail

t002
        ; NE - Z clear
        msr     cpsr_flg, 0
        bne     t003
        mov     r0, 2
        b       fail

t003
        ; CS - C set
        msr     cpsr_flg, FLAG_C
        bcs     t004
        mov     r0, 3
        b       fail

t004
        ; CC - C clear
        msr     cpsr_flg, 0
        bcc     t005
        mov     r0, 4
        b       fail

t005
        ; MI - N set
        msr     cpsr_flg, FLAG_N
        bmi     t006
        mov     r0, 5
        b       fail

t006
        ; PL - N clear
        msr     cpsr_flg, 0
        bpl     t007
        mov     r0, 6
        b       fail

t007
        ; VS - V set
        msr     cpsr_flg, FLAG_V
        bvs     t008
        mov     r0, 7
        b       fail

t008
        ; VC - V clear
        msr     cpsr_flg, 0
        bvc     t009
        mov     r0, 8
        b       fail

t009
        ; HI - C set and Z clear
        msr     cpsr_flg, FLAG_C
        bhi     t010
        mov     r0, 9
        b       fail

t010
        ; LS - C clear and Z set
        msr     cpsr_flg, FLAG_Z
        bls     t011
        mov     r0, 10
        b       fail

t011
        ; GE - N equals V
        msr     cpsr_flg, 0
        bge     t012
        mov     r0, 11
        b       fail

t012
        msr     cpsr_flg, FLAG_N or FLAG_V
        bge     t013
        mov     r0, 12
        b       fail

t013
        ; LT - N not equals to V
        msr     cpsr_flg, FLAG_N
        blt     t014
        mov     r0, 13
        b       fail

t014
        msr     cpsr_flg, FLAG_V
        blt     t015
        mov     r0, 14
        b       fail

t015
        ; GT - Z clear and (N equals V)
        msr     cpsr_flg, 0
        bgt     t016
        mov     r0, 15
        b       fail

t016
        msr     cpsr_flg, FLAG_N or FLAG_V
        bgt     t017
        mov     r0, 16
        b       fail

t017
        ; LE - Z set or (N not equal to V)
        msr     cpsr_flg, FLAG_Z
        ble     t018
        mov     r0, 17
        b       fail

t018
        msr     cpsr_flg, FLAG_N
        ble     t019
        mov     r0, 18
        b       fail

t019
        msr     cpsr_flg, FLAG_V
        ble     t020
        mov     r0, 19
        b       fail

t020
        ; AL - always
        bal     conditions_passed
        mov     r0, 20
        b       fail

fail
        ; Run an invalid SWI to abort.
        swi 100

conditions_passed
        swi 2
