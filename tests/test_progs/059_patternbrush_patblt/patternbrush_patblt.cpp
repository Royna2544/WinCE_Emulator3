#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_done = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        HDC mem = CreateCompatibleDC(dc);
        HBITMAP bmp = CreateCompatibleBitmap(dc, 8, 8);
        if (mem && bmp) {
            HBITMAP oldBmp = (HBITMAP)SelectObject(mem, bmp);
            RECT rc;
            SetRect(&rc, 0, 0, 8, 8);
            HBRUSH white = CreateSolidBrush(RGB(255,255,255));
            FillRect(mem, &rc, white);
            DeleteObject(white);
            SetRect(&rc, 0, 0, 4, 4);
            HBRUSH black = CreateSolidBrush(RGB(0,0,0));
            FillRect(mem, &rc, black);
            DeleteObject(black);
            SelectObject(mem, oldBmp);

            HBRUSH pattern = CreatePatternBrush(bmp);
            HBRUSH oldBrush = (HBRUSH)SelectObject(dc, pattern);
            PatBlt(dc, 0, 0, 80, 50, PATCOPY);
            SetBrushOrgEx(dc, 2, 2, 0);
            PatBlt(dc, 82, 0, 80, 50, PATCOPY);
            SelectObject(dc, oldBrush);
            DeleteObject(pattern);
            g_done = 1;
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
    wc.lpszClassName = L"FixturePatternBrushClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5901);
    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"pattern", WS_VISIBLE, 0, 0, 180, 70, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(5902);
    UpdateWindow(hwnd);
    if (!g_done) return FixtureFail(5903);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
