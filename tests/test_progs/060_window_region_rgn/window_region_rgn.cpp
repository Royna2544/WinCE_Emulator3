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
    wc.lpszClassName = L"FixtureWindowRegionClass";
    if (!RegisterClassW(&wc)) return FixtureFail(6001);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"region window", WS_VISIBLE, 0, 0, 120, 80, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(6002);

    HRGN region = CreateRectRgn(5, 5, 100, 60);
    if (!region) return FixtureFail(6003);

    if (!SetWindowRgn(hwnd, region, TRUE)) return FixtureFail(6004);

    HRGN out = CreateRectRgn(0, 0, 0, 0);
    if (!out) return FixtureFail(6005);

    int result = GetWindowRgn(hwnd, out);
    if (result == ERROR) return FixtureFail(6006);

    DeleteObject(out);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
