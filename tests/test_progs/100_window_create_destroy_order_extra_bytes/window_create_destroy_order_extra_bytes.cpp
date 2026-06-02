#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_order[8];
static DWORD g_count = 0;

static void Mark(DWORD v) {
    if (g_count < 8) g_order[g_count++] = v;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_CREATE) {
        Mark(1);
        SetWindowLongW(hwnd, 0, 0x12345678);
        return 0;
    }
    if (msg == WM_CLOSE) {
        Mark(2);
        DestroyWindow(hwnd);
        return 0;
    }
    if (msg == WM_DESTROY) {
        Mark(3);
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.cbWndExtra = sizeof(LONG);
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureCreateDestroyOrderClass";
    if (!RegisterClassW(&wc)) return FixtureFail(10001);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"create destroy", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(10002);

    if (GetWindowLongW(hwnd, 0) != 0x12345678) return FixtureFail(10003);

    SendMessageW(hwnd, WM_CLOSE, 0, 0);

    MSG msg;
    while (GetMessageW(&msg, 0, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    if (g_count < 3 || g_order[0] != 1 || g_order[1] != 2 || g_order[2] != 3) return FixtureFail(10004);

    return FIXTURE_OK;
}
