#include <windows.h>
#include "../common/fixture_status.h"
static DWORD g_count = 0;
static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 152) { ++g_count; return 0x098; }
    if (msg == WM_TIMER) { ++g_count; if (g_count > 20) { KillTimer(hwnd, 152); PostQuitMessage(0); } return 0; }
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps; HDC dc = BeginPaint(hwnd, &ps);
        RECT rc; GetClientRect(hwnd, &rc);
        HBRUSH b = CreateSolidBrush(RGB(152, 40, 80));
        FillRect(dc, &rc, b);
        DeleteObject(b);
        EndPaint(hwnd, &ps);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}
int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc; ZeroMemory(&wc, sizeof(wc)); wc.lpfnWndProc = WndProc; wc.hInstance = h; wc.lpszClassName = L"StressClass152";
    if (!RegisterClassW(&wc)) return FixtureFail(15201);
    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"stress", WS_VISIBLE, 0,0,160,100,0,0,h,0);
    if (!hwnd) return FixtureFail(15202);
    int i;
    for (i = 0; i < 200; ++i) PostMessageW(hwnd, WM_USER + 152, i, 0);
    SetTimer(hwnd, 152, 1, 0);
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
    if (g_count < 200) return FixtureFail(15203);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
