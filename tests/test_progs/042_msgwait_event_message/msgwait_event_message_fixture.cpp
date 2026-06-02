#include <windows.h>
#include "../common/fixture_status.h"

#ifndef QS_ALLINPUT
#define QS_ALLINPUT 0x04ff
#endif
#ifndef MWMO_INPUTAVAILABLE
#define MWMO_INPUTAVAILABLE 0x0004
#endif

static DWORD g_seen = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 42) {
        g_seen = (DWORD)wp;
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureMsgWaitClass";
    if (!RegisterClassW(&wc)) return FixtureFail(4201);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"msgwait", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(4202);

    if (!PostMessageW(hwnd, WM_USER + 42, 77, 0)) return FixtureFail(4203);

    DWORD r = MsgWaitForMultipleObjectsEx(0, 0, 1000, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    if (r == WAIT_TIMEOUT) return FixtureFail(4204);

    MSG msg;
    if (!PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) return FixtureFail(4205);
    TranslateMessage(&msg);
    DispatchMessageW(&msg);

    if (g_seen != 77) return FixtureFail(4206);

    HANDLE eventHandle = CreateEventW(0, TRUE, TRUE, 0);
    if (!eventHandle) return FixtureFail(4207);

    r = MsgWaitForMultipleObjectsEx(1, &eventHandle, 1000, QS_ALLINPUT, 0);
    if (r != WAIT_OBJECT_0) return FixtureFail(4208);

    CloseHandle(eventHandle);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
