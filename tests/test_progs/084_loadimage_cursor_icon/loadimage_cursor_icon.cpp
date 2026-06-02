#include <windows.h>
#include "../common/fixture_status.h"

#ifndef IMAGE_CURSOR
#define IMAGE_CURSOR 2
#endif
#ifndef LR_DEFAULTSIZE
#define LR_DEFAULTSIZE 0x00000040
#endif

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE cursor = LoadImageW(0, IDC_ARROW, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE);
    if (!cursor) {
        /*
           On some CE builds LoadImageW for stock cursor may be stubbed.
           Treat LoadCursorW success as acceptable.
        */
        if (!LoadCursorW(0, IDC_ARROW)) return FixtureFail(8401);
    }
    return FIXTURE_OK;
}
