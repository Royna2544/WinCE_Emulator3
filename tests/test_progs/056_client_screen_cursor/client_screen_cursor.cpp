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
    wc.lpszClassName = L"FixtureClientScreenCursorClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5601);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"coords", WS_VISIBLE, 10, 20, 100, 60, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(5602);

    POINT pt;
    pt.x = 7;
    pt.y = 9;

    POINT original = pt;
    if (!ClientToScreen(hwnd, &pt)) return FixtureFail(5603);
    if (!ScreenToClient(hwnd, &pt)) return FixtureFail(5604);
    if (pt.x != original.x || pt.y != original.y) return FixtureFail(5605);

    POINT cursor;
    if (!GetCursorPos(&cursor)) return FixtureFail(5606);

    SetCursor(LoadCursorW(0, IDC_ARROW));
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
