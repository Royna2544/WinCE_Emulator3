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
    wc.lpszClassName = L"FixtureGetWindowWalkClass";
    if (!RegisterClassW(&wc)) return FixtureFail(9901);

    HWND parent = CreateWindowExW(0, wc.lpszClassName, L"parent", WS_VISIBLE, 0, 0, 200, 120, 0, 0, h, 0);
    if (!parent) return FixtureFail(9902);

    HWND c1 = CreateWindowExW(0, wc.lpszClassName, L"c1", WS_CHILD | WS_VISIBLE, 0, 0, 50, 30, parent, (HMENU)1, h, 0);
    HWND c2 = CreateWindowExW(0, wc.lpszClassName, L"c2", WS_CHILD | WS_VISIBLE, 10, 10, 50, 30, parent, (HMENU)2, h, 0);
    HWND c3 = CreateWindowExW(0, wc.lpszClassName, L"c3", WS_CHILD | WS_VISIBLE, 20, 20, 50, 30, parent, (HMENU)3, h, 0);
    if (!c1 || !c2 || !c3) return FixtureFail(9903);

    HWND first = GetWindow(parent, GW_CHILD);
    if (!first) return FixtureFail(9904);

    HWND next = GetWindow(first, GW_HWNDNEXT);
    (void)next;

    HWND owner = GetWindow(parent, GW_OWNER);
    (void)owner;

    DestroyWindow(parent);
    return FIXTURE_OK;
}
