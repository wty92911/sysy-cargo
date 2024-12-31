  .text
  .global main
main:
  addi sp, sp, -112
main_0:
  li a0, 0
  sw a0, 0(sp)
  li t1, 1
  sw t1, 4(sp)
  li t0, 2
  sw t0, 8(sp)
  sw a0, 12(sp)
  sw a0, 12(sp)
  sw t1, 4(sp)
  lw t1, 12(sp)
  sw t0, 8(sp)
  li t0, 0
  or a0, t1, t0
  sw a0, 16(sp)
  sw t1, 12(sp)
  li t0, 0
  sub t1, a0, t0
  snez t1, t1
  sw t1, 20(sp)
  bnez t1, if_1
  j else_2
if_1:
  sw t1, 20(sp)
  li t1, 3
  sw t1, 4(sp)
  j if_end_3
else_2:
  sw a0, 16(sp)
  li a0, 3
  sw a0, 8(sp)
  li t0, 1
  sw t0, 24(sp)
  sw t0, 28(sp)
  sw a0, 8(sp)
  sw t1, 4(sp)
  li t1, 0
  or a0, t0, t1
  sw a0, 32(sp)
  sw t0, 28(sp)
  li t0, 0
  sub t1, a0, t0
  snez t1, t1
  sw t1, 36(sp)
  bnez t1, if_4
  j else_5
if_end_3:
  lw t0, 0(sp)
  sw t0, 40(sp)
  sw t0, 40(sp)
  sw t1, 36(sp)
  lw t1, 40(sp)
  sw a0, 32(sp)
  li a0, 0
  add t0, t1, a0
  sw t0, 44(sp)
  lw a0, 4(sp)
  sw a0, 48(sp)
  sw t1, 40(sp)
  add t1, t0, a0
  sw t1, 52(sp)
  sw t0, 44(sp)
  lw t0, 8(sp)
  sw t0, 56(sp)
  sw a0, 48(sp)
  add a0, t1, t0
  sw a0, 60(sp)
  sw a0, 60(sp)
  add sp, sp, 112
  ret
if_4:
  sw t1, 52(sp)
  li t1, 4
  sw t1, 4(sp)
  j if_end_6
else_5:
  sw a0, 60(sp)
  li a0, 4
  sw a0, 8(sp)
  j if_end_6
if_end_6:
  sw t1, 4(sp)
  lw t1, 24(sp)
  sw t1, 64(sp)
  sw t1, 64(sp)
  sw a0, 8(sp)
  lw a0, 64(sp)
  sw t0, 56(sp)
  li t0, 0
  sub t1, a0, t0
  seqz t1, t1
  sw t1, 68(sp)
  bnez t1, if_7
  j else_8
if_7:
  sw a0, 64(sp)
  li a0, 1
  add sp, sp, 112
  ret
else_8:
  sw t1, 68(sp)
  lw t1, 24(sp)
  sw t1, 72(sp)
  li a0, 0
  sub t0, t1, a0
  seqz t0, t0
  sw t0, 76(sp)
  lw a0, 24(sp)
  sw a0, 80(sp)
  sw t1, 72(sp)
  sw a0, 80(sp)
  li a0, 0
  sw t0, 76(sp)
  li t0, 1
  sub t1, a0, t0
  sw t1, 84(sp)
  sw t1, 84(sp)
  lw a0, 80(sp)
  lw t0, 84(sp)
  sub t1, a0, t0
  seqz t1, t1
  sw t1, 88(sp)
  sw t0, 84(sp)
  sw t1, 88(sp)
  lw t1, 76(sp)
  sw a0, 80(sp)
  li a0, 0
  sub t0, t1, a0
  snez t0, t0
  sw t0, 92(sp)
  sw t1, 76(sp)
  sw t0, 92(sp)
  lw t0, 88(sp)
  sub t1, t0, a0
  snez t1, t1
  sw t1, 96(sp)
  sw t0, 88(sp)
  lw t0, 92(sp)
  and a0, t0, t1
  sw a0, 100(sp)
  bnez a0, if_10
  j else_11
if_end_9:
  j if_end_3
if_10:
  sw a0, 100(sp)
  li a0, 2
  add sp, sp, 112
  ret
else_11:
  j if_end_12
if_end_12:
  j if_end_9
