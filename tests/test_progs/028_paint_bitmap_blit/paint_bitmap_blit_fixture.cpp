#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

#ifndef IMAGE_BITMAP
#define IMAGE_BITMAP 0
#endif
#ifndef LR_CREATEDIBSECTION
#define LR_CREATEDIBSECTION 0x00002000
#endif

static DWORD g_painted = 0;
static HBITMAP g_bitmap = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    switch (msg) {
    case WM_CREATE:
        InvalidateRect(hwnd, 0, TRUE);
        return 0;

    case WM_PAINT: {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);
        RECT rc;
        SetRect(&rc, 0, 0, 96, 64);
        HBRUSH bg = CreateSolidBrush(RGB(20, 40, 80));
        FillRect(dc, &rc, bg);
        DeleteObject(bg);

        HDC mem = CreateCompatibleDC(dc);
        if (mem && g_bitmap) {
            HBITMAP old = (HBITMAP)SelectObject(mem, g_bitmap);
            StretchBlt(dc, 8, 8, 32, 32, mem, 0, 0, 4, 4, SRCCOPY);
            SelectObject(mem, old);
        }
        if (mem) DeleteDC(mem);

        DrawTextW(dc, L"bmp", -1, &rc, DT_LEFT | DT_BOTTOM);
        EndPaint(hwnd, &ps);
        g_painted = 1;
        return 0;
    }
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    g_bitmap = (HBITMAP)LoadImageW(h, MAKEINTRESOURCEW(IDB_TEST_4X4), IMAGE_BITMAP, 0, 0, LR_CREATEDIBSECTION);
    if (!g_bitmap) return FixtureFail(2801);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixturePaintBitmapBlitClass";
    if (!RegisterClassW(&wc)) return FixtureFail(2802);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"paint bitmap blit", WS_VISIBLE, 0, 0, 96, 64, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(2803);

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

    if (!g_painted) return FixtureFail(2804);

    DestroyWindow(hwnd);
    DeleteObject(g_bitmap);
    return FIXTURE_OK;
}
