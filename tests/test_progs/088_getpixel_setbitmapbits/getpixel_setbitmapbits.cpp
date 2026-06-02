#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HDC screen = GetDC(0);
    if (!screen) return FixtureFail(8801);

    HDC mem = CreateCompatibleDC(screen);
    HBITMAP bmp = CreateCompatibleBitmap(screen, 2, 2);
    if (!mem || !bmp) return FixtureFail(8802);

    HBITMAP old = (HBITMAP)SelectObject(mem, bmp);

    RECT rc;
    SetRect(&rc, 0, 0, 2, 2);
    HBRUSH brush = CreateSolidBrush(RGB(10, 20, 30));
    FillRect(mem, &rc, brush);
    DeleteObject(brush);

    COLORREF px = GetPixel(mem, 0, 0);
    if (px == CLR_INVALID) return FixtureFail(8803);

    BYTE bits[16];
    DWORD i;
    for (i = 0; i < sizeof(bits); ++i) bits[i] = (BYTE)(i * 7);
    SetBitmapBits(bmp, sizeof(bits), bits);

    SelectObject(mem, old);
    DeleteObject(bmp);
    DeleteDC(mem);
    ReleaseDC(0, screen);
    return FIXTURE_OK;
}
