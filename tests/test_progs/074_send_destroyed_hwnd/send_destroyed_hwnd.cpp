#include <windows.h>
#include "../common/fixture_status.h"

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureDestroyedHwndClass";
    if (!RegisterClassW(&wc)) return FixtureFail(7401);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"destroyed", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(7402);

    DestroyWindow(hwnd);

    SetLastError(0);
    LRESULT r = SendMessageW(hwnd, WM_USER + 74, 0, 0);
    (void)r;

    SetLastError(0);
    if (SetWindowTextW(hwnd, L"bad")) return FixtureFail(7403);

    return FIXTURE_OK;
}
