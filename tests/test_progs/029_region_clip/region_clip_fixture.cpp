#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_done = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        HRGN r1 = CreateRectRgn(10, 10, 80, 50);
        HRGN r2 = CreateRectRgn(40, 20, 110, 70);
        HRGN combined = CreateRectRgn(0, 0, 0, 0);

        if (!r1 || !r2 || !combined) { EndPaint(hwnd, &ps); return 0; }

        CombineRgn(combined, r1, r2, RGN_OR);
        SelectClipRgn(dc, combined);

        RECT fill;
        SetRect(&fill, 0, 0, 120, 80);
        HBRUSH brush = CreateSolidBrush(RGB(100, 20, 180));
        FillRect(dc, &fill, brush);
        DeleteObject(brush);

        RECT box;
        GetClipBox(dc, &box);

        POINT pt;
        pt.x = 20;
        pt.y = 20;

        if (PtInRegion(combined, pt.x, pt.y) && RectInRegion(combined, &box)) {
            g_done = 1;
        }

        SelectClipRgn(dc, 0);
        DeleteObject(combined);
        DeleteObject(r2);
        DeleteObject(r1);
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
    wc.lpszClassName = L"FixtureRegionClipClass";
    if (!RegisterClassW(&wc)) return FixtureFail(2901);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"region clip", WS_VISIBLE, 0, 0, 120, 80, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(2902);

    InvalidateRect(hwnd, 0, TRUE);
    UpdateWindow(hwnd);

    MSG msg;
    DWORD spins = 0;
    while (!g_done && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }

    if (!g_done) return FixtureFail(2903);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
