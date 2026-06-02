#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_painted = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        BITMAPINFO bmi;
        ZeroMemory(&bmi, sizeof(bmi));
        bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
        bmi.bmiHeader.biWidth = 2;
        bmi.bmiHeader.biHeight = -2;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        void* bits = 0;
        HBITMAP dib = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &bits, 0, 0);
        if (dib && bits) {
            DWORD* px = (DWORD*)bits;
            px[0] = 0x00ff0000;
            px[1] = 0x0000ff00;
            px[2] = 0x000000ff;
            px[3] = 0x00ffffff;

            HDC mem = CreateCompatibleDC(dc);
            HBITMAP old = (HBITMAP)SelectObject(mem, dib);

            StretchBlt(dc, 0, 0, 32, 32, mem, 0, 0, 2, 2, SRCCOPY);
            StretchDIBits(dc, 40, 0, 32, 32, 0, 0, 2, 2, bits, &bmi, DIB_RGB_COLORS, SRCCOPY);
            SetDIBitsToDevice(dc, 0, 40, 2, 2, 0, 0, 0, 2, bits, &bmi, DIB_RGB_COLORS);

            SelectObject(mem, old);
            DeleteDC(mem);
            DeleteObject(dib);
            g_painted = 1;
        }

        EndPaint(hwnd, &ps);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureDibStretchPaintClass";
    if (!RegisterClassW(&wc)) return FixtureFail(4501);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"dib stretch", WS_VISIBLE, 0, 0, 90, 70, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(4502);

    InvalidateRect(hwnd, 0, TRUE);
    UpdateWindow(hwnd);

    MSG msg;
    DWORD spins = 0;
    while (!g_painted && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }

    if (!g_painted) return FixtureFail(4503);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
