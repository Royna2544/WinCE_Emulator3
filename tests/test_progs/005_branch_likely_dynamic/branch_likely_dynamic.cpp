#include <windows.h>
#include "../common/fixture_status.h"
#include "../common/mips_dyn_code.h"

/*
   Runtime-generated MIPS little-endian branch-likely tests.

   Branch-likely rule:
     taken:     execute delay slot
     not taken: annul/skip delay slot

   These opcodes are intentionally old MIPS II/III style and may expose CPU backend gaps.
*/

static int TestBgezlTaken() {
    /*
       addiu v0, zero, 0
       bgezl zero, target       ; taken because zero >= 0
       addiu v0, zero, 2        ; delay slot must execute
     target:
       jr ra
       nop

       Expected return: 2
    */
    const DWORD code[] = {
        0x24020000u, // addiu v0, zero, 0
        0x04030001u, // bgezl zero, +1
        0x24020002u, // addiu v0, zero, 2
        0x03e00008u, // jr ra
        0x00000000u, // nop
    };

    DWORD allocError = 0;
    int result = RunMipsWords(code, sizeof(code) / sizeof(code[0]), &allocError);
    if (result == -1 && allocError) {
        return -100;
    }
    return result;
}

static int TestBltzlNotTakenAnnulsDelaySlot() {
    /*
       addiu v0, zero, 5
       bltzl zero, taken        ; not taken because zero < 0 is false
       addiu v0, zero, 99       ; delay slot must be annulled/skipped
       jr ra
       nop
     taken:
       addiu v0, zero, 7
       jr ra
       nop

       Expected return: 5
       If the delay slot incorrectly executes, return is 99.
       If branch is incorrectly taken, return is 7.
    */
    const DWORD code[] = {
        0x24020005u, // addiu v0, zero, 5
        0x04020003u, // bltzl zero, +3 to taken
        0x24020063u, // addiu v0, zero, 99
        0x03e00008u, // jr ra
        0x00000000u, // nop
        0x24020007u, // taken: addiu v0, zero, 7
        0x03e00008u, // jr ra
        0x00000000u, // nop
    };

    DWORD allocError = 0;
    int result = RunMipsWords(code, sizeof(code) / sizeof(code[0]), &allocError);
    if (result == -1 && allocError) {
        return -100;
    }
    return result;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    int taken = TestBgezlTaken();
    if (taken != 2) {
        return FixtureFail(1);
    }

    int notTaken = TestBltzlNotTakenAnnulsDelaySlot();
    if (notTaken != 5) {
        return FixtureFail(2);
    }

    return FIXTURE_OK;
}
