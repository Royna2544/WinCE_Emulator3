#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_done = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);
        RECT rc;
        SetRect(&rc, 0, 0, 160, 80);

        SetBkColor(dc, RGB(0, 0, 120));
        SetBkMode(dc, OPAQUE);
        SetTextColor(dc, RGB(255, 255, 255));

        DrawTextW(dc, L"center\nwordbreak", -1, &rc, DT_CENTER | DT_WORDBREAK);
        ExtTextOutW(dc, 4, 50, ETO_OPAQUE, &rc, L"opaque", 6, 0);
        g_done = 1;

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
    wc.lpszClassName = L"FixtureDrawTextFlagsClass";
    if (!RegisterClassW(&wc)) return FixtureFail(9201);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"text flags", WS_VISIBLE, 0, 0, 160, 80, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(9202);
    UpdateWindow(hwnd);
    if (!g_done) return FixtureFail(9203);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
