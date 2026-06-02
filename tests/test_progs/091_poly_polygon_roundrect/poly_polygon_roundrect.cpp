#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_done = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        POINT poly[4];
        poly[0].x = 10; poly[0].y = 10;
        poly[1].x = 50; poly[1].y = 15;
        poly[2].x = 40; poly[2].y = 50;
        poly[3].x = 5;  poly[3].y = 45;

        HBRUSH brush = CreateSolidBrush(RGB(120, 40, 180));
        HBRUSH oldBrush = (HBRUSH)SelectObject(dc, brush);
        Polygon(dc, poly, 4);
        SelectObject(dc, oldBrush);
        DeleteObject(brush);

        Polyline(dc, poly, 4);
        RoundRect(dc, 60, 10, 120, 60, 12, 12);
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
    wc.lpszClassName = L"FixturePolyRoundClass";
    if (!RegisterClassW(&wc)) return FixtureFail(9101);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"poly", WS_VISIBLE, 0, 0, 140, 80, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(9102);
    UpdateWindow(hwnd);
    if (!g_done) return FixtureFail(9103);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
