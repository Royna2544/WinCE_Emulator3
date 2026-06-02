#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_down = 0;
static DWORD g_move = 0;
static DWORD g_up = 0;
static DWORD g_lastX = 0;
static DWORD g_lastY = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_LBUTTONDOWN) {
        g_down = 1;
        SetCapture(hwnd);
        g_lastX = LOWORD(lp);
        g_lastY = HIWORD(lp);
        return 0;
    }
    if (msg == WM_MOUSEMOVE) {
        g_move = 1;
        g_lastX = LOWORD(lp);
        g_lastY = HIWORD(lp);
        return 0;
    }
    if (msg == WM_LBUTTONUP) {
        g_up = 1;
        ReleaseCapture();
        g_lastX = LOWORD(lp);
        g_lastY = HIWORD(lp);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureMouseCaptureClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5501);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"mouse", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(5502);

    PostMessageW(hwnd, WM_LBUTTONDOWN, MK_LBUTTON, MAKELPARAM(10, 11));
    PostMessageW(hwnd, WM_MOUSEMOVE, MK_LBUTTON, MAKELPARAM(20, 21));
    PostMessageW(hwnd, WM_LBUTTONUP, 0, MAKELPARAM(30, 31));

    MSG msg;
    DWORD spins = 0;
    while (!g_up && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }

    if (!g_down || !g_move || !g_up) return FixtureFail(5503);
    if (g_lastX != 30 || g_lastY != 31) return FixtureFail(5504);
    if (GetCapture() != 0) return FixtureFail(5505);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
