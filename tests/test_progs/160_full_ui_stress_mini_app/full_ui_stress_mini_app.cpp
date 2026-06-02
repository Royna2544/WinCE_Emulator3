#include <windows.h>
#include "../common/fixture_status.h"
static DWORD g_count = 0;
static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 160) { ++g_count; return 0x0a0; }
    if (msg == WM_TIMER) { ++g_count; if (g_count > 20) { KillTimer(hwnd, 160); PostQuitMessage(0); } return 0; }
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps; HDC dc = BeginPaint(hwnd, &ps);
        RECT rc; GetClientRect(hwnd, &rc);
        HBRUSH b = CreateSolidBrush(RGB(160, 40, 80));
        FillRect(dc, &rc, b);
        DeleteObject(b);
        EndPaint(hwnd, &ps);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}
int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc; ZeroMemory(&wc, sizeof(wc)); wc.lpfnWndProc = WndProc; wc.hInstance = h; wc.lpszClassName = L"StressClass160";
    if (!RegisterClassW(&wc)) return FixtureFail(16001);
    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"stress", WS_VISIBLE, 0,0,160,100,0,0,h,0);
    if (!hwnd) return FixtureFail(16002);
    int i;
    for (i = 0; i < 200; ++i) PostMessageW(hwnd, WM_USER + 160, i, 0);
    SetTimer(hwnd, 160, 1, 0);
    MSG msg; DWORD spins = 0;
    while (g_count < 220 && spins++ < 3000) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        InvalidateRect(hwnd, 0, FALSE);
        UpdateWindow(hwnd);
        Sleep(1);
    }
    if (g_count < 200) return FixtureFail(16003);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
