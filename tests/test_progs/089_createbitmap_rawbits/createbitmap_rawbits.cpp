#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    WORD bits[16];
    int i;
    for (i = 0; i < 16; ++i) bits[i] = (WORD)(0x7fff - i);

    HBITMAP bmp = CreateBitmap(4, 4, 1, 16, bits);
    if (!bmp) return FixtureFail(8901);

    BITMAP bm;
    ZeroMemory(&bm, sizeof(bm));
    if (!GetObjectW(bmp, sizeof(bm), &bm)) return FixtureFail(8902);
    if (bm.bmWidth != 4 || bm.bmHeight != 4) return FixtureFail(8903);

    DeleteObject(bmp);
    return FIXTURE_OK;
}
