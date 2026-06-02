.text
    .set noreorder
    .globl fixture_bgezl_taken
fixture_bgezl_taken:
    addiu   $v0, $zero, 0
    bgezl   $zero, 1f
    addiu   $v0, $zero, 2      # executes because branch is taken
1:
    jr      $ra
    nop

    .globl fixture_bltzl_not_taken
fixture_bltzl_not_taken:
    addiu   $v0, $zero, 5
    bltzl   $zero, 2f
    addiu   $v0, $zero, 99     # must be annulled because branch is not taken
    jr      $ra
    nop
2:
    addiu   $v0, $zero, 7
    jr      $ra
    nop
