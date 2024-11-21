128: a
129: b
130: xor
131: out
133: mask = 1
134: 1

; xor
lda 128
not ; not a
and 129 ; and ~a b
sta 132

lda 129
not ; not b
and 128 ; and ~b a
or 132 ; or (and ~b a) (and ~a b)
sta 130 ; mem[xor] <- 
; end xor -> 130

; while mask != 0
loop:
lda 133
jz end
; and xor mask
and 130
jz noadd
; out = out + 1
lda 131
add 134
sta 131
noadd:
lda 133
add 133
sta 133
jmp loop

end:
hlt