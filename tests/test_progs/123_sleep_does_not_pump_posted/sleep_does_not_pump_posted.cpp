#include <windows.h>
#include "../common/fixture_status.h"
static DWORD g_seen = 0;
static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 123) { g_seen = 1; return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}
int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc; ZeroMemory(&wc, sizeof(wc)); wc.lpfnWndProc = WndProc; wc.hInstance = h; wc.lpszClassName = L"SleepNoPump123";
    RegisterClassW(&wc);
    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"sleep", WS_VISIBLE, 0,0,80,40,0,0,h,0);
    if (!hwnd) return FixtureFail(12301);
    PostMessageW(hwnd, WM_USER + 123, 0, 0);
    Sleep(30);
    if (g_seen) return FixtureFail(12302);
    MSG msg;
    if (!PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) return FixtureFail(12303);
    DispatchMessageW(&msg);
    if (!g_seen) return FixtureFail(12304);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
