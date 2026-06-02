#include <windows.h>
#include "../common/fixture_status.h"
int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HDC screen = GetDC(0);
    if (!screen) return FixtureFail(14601);
    int i;
    for (i = 0; i < 300; ++i) {
        HDC mem = CreateCompatibleDC(screen);
        HBITMAP bmp = CreateCompatibleBitmap(screen, 16, 16);
        HBRUSH brush = CreateSolidBrush(RGB(i & 255, (i*3) & 255, (i*7) & 255));
        HPEN pen = CreatePen(PS_SOLID, 1, RGB((i*5)&255, 0, 80));
        HRGN rgn = CreateRectRgn(0, 0, 10 + (i % 20), 10 + (i % 20));
        if (!mem || !bmp || !brush || !pen || !rgn) return FixtureFail(14602);
        RECT rc; SetRect(&rc, 0, 0, 16, 16);
        FillRect(mem, &rc, brush);
        DeleteObject(rgn);
        DeleteObject(pen);
        DeleteObject(brush);
        DeleteObject(bmp);
        DeleteDC(mem);
    }
    ReleaseDC(0, screen);
    return FIXTURE_OK;
}
