#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureCoordinateWindow";

static LRESULT CALLBACK FixtureWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = FixtureWndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = kClassName;
    if (!RegisterClassW(&wc)) {
        return FixtureFail(1);
    }

    HWND parent = CreateWindowExW(0, kClassName, L"parent", WS_VISIBLE, 10, 20, 200, 100, 0, 0, hInstance, 0);
    HWND child = CreateWindowExW(0, kClassName, L"child", WS_CHILD | WS_VISIBLE, 5, 7, 50, 30, parent, (HMENU)9, hInstance, 0);
    if (!parent || !child) {
        return FixtureFail(2);
    }

    POINT pt;
    pt.x = 3;
    pt.y = 4;
    if (!ClientToScreen(child, &pt)) {
        return FixtureFail(3);
    }
    if (pt.x != 18 || pt.y != 31) {
        return FixtureFail(4);
    }
    if (!ScreenToClient(child, &pt)) {
        return FixtureFail(5);
    }
    if (pt.x != 3 || pt.y != 4) {
        return FixtureFail(6);
    }

    POINT pts[2];
    pts[0].x = 0;
    pts[0].y = 0;
    pts[1].x = 10;
    pts[1].y = 10;
    DWORD packed = MapWindowPoints(child, parent, pts, 2);
    if (LOWORD(packed) != 5 || HIWORD(packed) != 7) {
        return FixtureFail(7);
    }
    if (pts[0].x != 5 || pts[0].y != 7 || pts[1].x != 15 || pts[1].y != 17) {
        return FixtureFail(8);
    }

    DestroyWindow(child);
    DestroyWindow(parent);
    return FIXTURE_OK;
}
