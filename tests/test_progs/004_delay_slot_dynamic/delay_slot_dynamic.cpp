#include <windows.h>
#include "../common/fixture_status.h"
#include "../common/mips_dyn_code.h"

/*
   Runtime-generated MIPS little-endian code.

   Expected behavior:
     ordinary branch delay slot ALWAYS executes.

   Code:
     addiu v0, zero, 0
     beq   zero, zero, target
     addiu v0, zero, 1      ; delay slot, must execute
   target:
     jr    ra
     nop

   Expected return: 1
*/

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    const DWORD code[] = {
        0x24020000u, // addiu v0, zero, 0
        0x10000001u, // beq zero, zero, +1 instruction to target
        0x24020001u, // addiu v0, zero, 1
        0x03e00008u, // jr ra
        0x00000000u, // nop
    };

    DWORD allocError = 0;
    int result = RunMipsWords(code, sizeof(code) / sizeof(code[0]), &allocError);
    if (result == -1 && allocError) {
        return FixtureFail(1);
    }

    return result == 1 ? FIXTURE_OK : FixtureFail(2);
}
