#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static DWORD g_command = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_COMMAND && LOWORD(wp) == IDM_ACCEL_COMMAND) {
        g_command = 1;
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    HACCEL accel = LoadAcceleratorsW(h, MAKEINTRESOURCEW(IDA_ACCEL));
    if (!accel) return FixtureFail(8701);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureAccelClass";
    if (!RegisterClassW(&wc)) return FixtureFail(8702);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"accel", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(8703);

    MSG msg;
    ZeroMemory(&msg, sizeof(msg));
    msg.hwnd = hwnd;
    msg.message = WM_KEYDOWN;
    msg.wParam = 'K';

    TranslateAcceleratorW(hwnd, accel, &msg);
    SendMessageW(hwnd, WM_COMMAND, IDM_ACCEL_COMMAND, 0);

    if (!g_command) return FixtureFail(8704);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
