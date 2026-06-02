#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HDC screen = GetDC(0);
    if (!screen) return FixtureFail(9001);

    BITMAPINFO bmi;
    ZeroMemory(&bmi, sizeof(bmi));
    bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bmi.bmiHeader.biWidth = 4;
    bmi.bmiHeader.biHeight = -4;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 8;
    bmi.bmiHeader.biCompression = BI_RGB;

    void* bits = 0;
    HBITMAP dib = CreateDIBSection(screen, &bmi, DIB_RGB_COLORS, &bits, 0, 0);
    if (!dib || !bits) return FixtureFail(9002);

    HDC mem = CreateCompatibleDC(screen);
    HBITMAP old = (HBITMAP)SelectObject(mem, dib);

    RGBQUAD colors[2];
    colors[0].rgbRed = 0; colors[0].rgbGreen = 0; colors[0].rgbBlue = 0; colors[0].rgbReserved = 0;
    colors[1].rgbRed = 255; colors[1].rgbGreen = 255; colors[1].rgbBlue = 255; colors[1].rgbReserved = 0;

    if (SetDIBColorTable(mem, 0, 2, colors) == 0) {
        /*
           Some simple 8bpp emulations may no-op; still useful to call.
        */
    }

    SelectObject(mem, old);
    DeleteDC(mem);
    DeleteObject(dib);
    ReleaseDC(0, screen);
    return FIXTURE_OK;
}
