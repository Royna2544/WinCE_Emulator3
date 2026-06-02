#include <windows.h>
#include <setjmp.h>
#include "../common/fixture_status.h"

static jmp_buf g_jump;
static volatile DWORD g_marker = 0;

static void JumpAway() {
    g_marker = 0x12345678;
    longjmp(g_jump, 42);
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    volatile DWORD localMarker = 0x11112222;

    int value = setjmp(g_jump);
    if (value == 0) {
        JumpAway();
        return FixtureFail(1);
    }

    if (value != 42) {
        return FixtureFail(2);
    }

    if (g_marker != 0x12345678) {
        return FixtureFail(3);
    }

    if (localMarker != 0x11112222) {
        return FixtureFail(4);
    }

    return FIXTURE_OK;
}
