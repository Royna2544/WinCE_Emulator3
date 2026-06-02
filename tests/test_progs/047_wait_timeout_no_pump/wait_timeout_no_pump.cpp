#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_seen = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 47) {
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
    wc.lpszClassName = L"FixtureWaitTimeoutNoPumpClass";
    if (!RegisterClassW(&wc)) return FixtureFail(4701);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"wait timeout no pump", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(4702);

    HANDLE eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!eventHandle) return FixtureFail(4703);

    if (!PostMessageW(hwnd, WM_USER + 47, 123, 0)) return FixtureFail(4704);

    DWORD wait = WaitForSingleObject(eventHandle, 25);
    if (wait != WAIT_TIMEOUT) return FixtureFail(4705);

    /*
       Plain WaitForSingleObject must not dispatch posted window messages.
       The posted message should still be waiting here.
    */
    if (g_seen != 0) return FixtureFail(4706);

    MSG msg;
    if (!PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) return FixtureFail(4707);
    TranslateMessage(&msg);
    DispatchMessageW(&msg);

    if (g_seen != 123) return FixtureFail(4708);

    CloseHandle(eventHandle);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
