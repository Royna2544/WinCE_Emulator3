.text
    .set noreorder
    .globl fixture_delay_slot
fixture_delay_slot:
    addiu   $v0, $zero, 0
    beq     $zero, $zero, 1f
    addiu   $v0, $zero, 1      # ordinary delay slot: must execute
1:
    jr      $ra
    nop
