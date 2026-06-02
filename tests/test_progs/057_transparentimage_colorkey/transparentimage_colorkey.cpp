#include <windows.h>
#include "../common/fixture_status.h"

#ifndef CLR_INVALID
#define CLR_INVALID 0xffffffff
#endif

static DWORD g_painted = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        HDC mem = CreateCompatibleDC(dc);
        HBITMAP bmp = CreateCompatibleBitmap(dc, 16, 16);
        if (mem && bmp) {
            HBITMAP old = (HBITMAP)SelectObject(mem, bmp);
            RECT rc;
            SetRect(&rc, 0, 0, 16, 16);

            HBRUSH magenta = CreateSolidBrush(RGB(255, 0, 255));
            FillRect(mem, &rc, magenta);
            DeleteObject(magenta);

            HBRUSH green = CreateSolidBrush(RGB(0, 255, 0));
            SetRect(&rc, 4, 4, 12, 12);
            FillRect(mem, &rc, green);
            DeleteObject(green);

            TransparentImage(dc, 4, 4, 16, 16, mem, 0, 0, 16, 16, RGB(255, 0, 255));
            SelectObject(mem, old);
            g_painted = 1;
        }

        if (bmp) DeleteObject(bmp);
        if (mem) DeleteDC(mem);
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
    wc.lpszClassName = L"FixtureTransparentImageClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5701);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"transparent", WS_VISIBLE, 0, 0, 80, 60, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(5702);

    InvalidateRect(hwnd, 0, TRUE);
    UpdateWindow(hwnd);

    if (!g_painted) return FixtureFail(5703);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
