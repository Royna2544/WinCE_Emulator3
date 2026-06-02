#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_paints = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);
        RECT rc;
        SetRect(&rc, 0, 0, 80, 40);
        HBRUSH brush = CreateSolidBrush(RGB(30 + g_paints * 20, 50, 70));
        FillRect(dc, &rc, brush);
        DeleteObject(brush);
        EndPaint(hwnd, &ps);
        ++g_paints;
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureInvalidateRepaintClass";
    if (!RegisterClassW(&wc)) return FixtureFail(3301);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"invalidate", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(3302);

    RECT smallRect;
    SetRect(&smallRect, 5, 5, 20, 20);

    InvalidateRect(hwnd, &smallRect, FALSE);
    UpdateWindow(hwnd);
    ValidateRect(hwnd, &smallRect);

    InvalidateRect(hwnd, 0, TRUE);
    UpdateWindow(hwnd);

    if (g_paints < 2) return FixtureFail(3303);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
