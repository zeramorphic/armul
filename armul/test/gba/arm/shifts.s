; Tests for shift operations

;! halts 200

FLAG_N equ 1 lsl 31
FLAG_Z equ 1 lsl 30
FLAG_C equ 1 lsl 29
FLAG_V equ 1 lsl 28

t150
        ; Logical shift left
        mov     r0, 1
        lsl     r0, r0, 6
        cmp     r0, 64
        bne     f150

        b       t151

f150
        swi     150

t151
        ; Logical shift left carry
        mov     r0, 1
        lsls    r0, r0, 31
        bcs     f151

        mov     r0, 2
        lsls    r0, r0, 31
        bcc     f151

        b       t152

f151
        swi     151

t152
        ; Logical shift left by 32
        mov     r0, 1
        mov     r1, 32
        lsls    r0, r0, r1
        bne     f152
        bcc     f152

        b       t153

f152
        swi     152

t153
        ; Logical shift left by greater 32
        mov     r0, 1
        mov     r1, 33
        lsls    r0, r0, r1
        bne     f153
        bcs     f153

        b       t154

f153
        swi     153

t154
        ; Logical shift right
        mov     r0, 64
        lsr     r0, r0, 6
        cmp     r0, 1
        bne     f154

        b       t155

f154
        swi     154

t155
        ; Logical shift right carry
        mov     r0, 2
        lsrs    r0, r0, 1
        bcs     f155

        mov     r0, 1
        lsrs    r0, r0, 1
        bcc     f155

        b       t156

f155
        swi     155

t156
        ; Logical shift right special
        mov     r0, 1
        lsrs    r0, r0, 32
        bne     f156
        bcs     f156

        mov     r0, 1 lsl 31
        lsrs    r0, r0, 32
        bne     f156
        bcc     f156

        b       t157

f156
        swi     156

t157
        ; Logical shift right by greater 32
        mov     r0, 1 lsl 31
        mov     r1, 33
        lsrs    r0, r0, r1
        bne     f157
        bcs     f157

        b       t158

f157
        swi     157

t158
        ; Arithmetic shift right
        mov     r0, 64
        asr     r0, r0, 6
        cmp     r0, 1
        bne     f158

        mov     r0, 1 lsl 31
        asr     r0, r0, 31
        mvn     r1, 0
        cmp     r1, r0
        bne     f158

        b       t159

f158
        swi     158

t159
        ; Arithmetic shift right carry
        mov     r0, 2
        asrs    r0, r0, 1
        bcs     f159

        mov     r0, 1
        asrs    r0, r0, 1
        bcc     f159

        b       t160

f159
        swi     159

t160
        ; Arithmetic shift right special
        mov     r0, 1
        asrs    r0, r0, 32
        bne     f160
        bcs     f160

        mov     r0, 1 lsl 31
        asrs    r0, r0, 32
        bcc     f160
        mvn     r1, 0
        cmp     r1, r0
        bne     f160

        b       t161

f160
        swi     160

t161
        ; Rotate right
        mov     r0, 1
        ror     r0, r0, 1
        cmp     r0, 1 lsl 31
        bne     f161

        b       t162

f161
        swi     161

t162
        ; Rotate right carry
        mov     r0, 2
        rors    r0, r0, 1
        bcs     f162

        mov     r0, 1
        rors    r0, r0, 1
        bcc     f162

        b       t163

f162
        swi     162

t163
        ; Rotate right special
        msr     cpsr_flg, FLAG_C
        mov     r0, 1
        rrxs    r0, r0
        bcc     f163
        bpl     f163

        msr     cpsr_flg, 0
        mov     r0, 1
        rrxs    r0, r0
        bcc     f163
        bne     f163

        b       t164

f163
        swi     163

t164
        ; Rotate right by 32
        mov     r0, 1 lsl 31
        mov     r1, 32
        rors    r0, r0, r1
        bcc     f164
        cmp     r0, 1 lsl 31
        bne     f164

        b       t165

f164
        swi     164

t165
        ; Rotate right by greater 32
        mov     r0, 2
        mov     r1, 33
        ror     r0, r0, r1
        cmp     r0, 1
        bne     f165

        b       t166

f165
        swi     165

t166
        ; Shift by 0 register value
        msr     cpsr_flg, FLAG_C
        mov     r0, 1
        mov     r1, 0
        lsls    r0, r0, r1
        lsrs    r0, r0, r1
        asrs    r0, r0, r1
        rors    r0, r0, r1
        bcc     f166
        cmp     r0, 1
        bne     f166

        b       t167

f166
        swi     166

t167
        ; Shift saved in lowest byte
        mov     r0, 1
        mov     r1, 0xF10
        lsl     r0, r0, r1
        cmp     r0, 1 lsl 16
        bne     f167

        b       t168

f167
        swi     167

t168
        ; Logical shift right by 32
        mov     r0, 1 lsl 31
        mov     r1, 32
        lsr     r0, r0, r1
        bcc     f168

        b       shifts_passed

f168
        swi     168

shifts_passed
        swi     2
