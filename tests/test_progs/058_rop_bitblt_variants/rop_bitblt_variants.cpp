#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_done = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        HDC mem = CreateCompatibleDC(dc);
        HBITMAP bmp = CreateCompatibleBitmap(dc, 32, 32);
        if (mem && bmp) {
            HBITMAP old = (HBITMAP)SelectObject(mem, bmp);
            RECT rc;
            SetRect(&rc, 0, 0, 32, 32);
            HBRUSH brush = CreateSolidBrush(RGB(30, 200, 80));
            FillRect(mem, &rc, brush);
            DeleteObject(brush);

            BitBlt(dc, 0, 0, 32, 32, mem, 0, 0, SRCCOPY);
            BitBlt(dc, 34, 0, 32, 32, mem, 0, 0, SRCINVERT);
            BitBlt(dc, 68, 0, 32, 32, mem, 0, 0, SRCPAINT);
            BitBlt(dc, 102, 0, 32, 32, mem, 0, 0, SRCAND);
            PatBlt(dc, 0, 34, 32, 16, DSTINVERT);

            SelectObject(mem, old);
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
    wc.lpszClassName = L"FixtureRopBltClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5801);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"rop", WS_VISIBLE, 0, 0, 150, 80, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(5802);
    UpdateWindow(hwnd);
    if (!g_done) return FixtureFail(5803);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
