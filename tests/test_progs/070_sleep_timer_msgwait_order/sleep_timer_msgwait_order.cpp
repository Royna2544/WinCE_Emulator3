#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_msg = 0;
static DWORD g_timer = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 70) { g_msg = 1; return 0; }
    if (msg == WM_TIMER) { g_timer = 1; KillTimer(hwnd, 70); return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureSleepTimerOrderClass";
    if (!RegisterClassW(&wc)) return FixtureFail(7001);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"sleep timer", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(7002);

    SetTimer(hwnd, 70, 10, 0);
    PostMessageW(hwnd, WM_USER + 70, 0, 0);

    Sleep(30);

    if (g_msg || g_timer) return FixtureFail(7003);

    MSG msg;
    DWORD spins = 0;
    while ((!g_msg || !g_timer) && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }

    if (!g_msg || !g_timer) return FixtureFail(7004);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
