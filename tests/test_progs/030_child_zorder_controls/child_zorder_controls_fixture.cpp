#include <windows.h>
#include "../common/fixture_status.h"

static LRESULT CALLBACK BasicWndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = BasicWndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureChildZOrderClass";
    if (!RegisterClassW(&wc)) return FixtureFail(3001);

    HWND parent = CreateWindowExW(0, wc.lpszClassName, L"parent", WS_VISIBLE, 0, 0, 200, 120, 0, 0, h, 0);
    if (!parent) return FixtureFail(3002);

    HWND c1 = CreateWindowExW(0, wc.lpszClassName, L"child1", WS_CHILD | WS_VISIBLE, 5, 5, 60, 40, parent, (HMENU)101, h, 0);
    HWND c2 = CreateWindowExW(0, wc.lpszClassName, L"child2", WS_CHILD | WS_VISIBLE, 20, 20, 60, 40, parent, (HMENU)102, h, 0);
    if (!c1 || !c2) return FixtureFail(3003);

    if (GetParent(c1) != parent || GetParent(c2) != parent) return FixtureFail(3004);
    if (GetDlgItem(parent, 101) != c1) return FixtureFail(3005);
    if (GetDlgCtrlID(c2) != 102) return FixtureFail(3006);

    EnableWindow(c2, FALSE);
    if (IsWindowEnabled(c2)) return FixtureFail(3007);
    EnableWindow(c2, TRUE);
    if (!IsWindowEnabled(c2)) return FixtureFail(3008);

    MoveWindow(c1, 8, 9, 50, 30, TRUE);
    RECT r;
    GetWindowRect(c1, &r);
    if (r.right <= r.left || r.bottom <= r.top) return FixtureFail(3009);

    SetWindowPos(c1, c2, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
    ShowWindow(c1, SW_HIDE);
    if (IsWindowVisible(c1)) return FixtureFail(3010);
    ShowWindow(c1, SW_SHOW);
    if (!IsWindowVisible(c1)) return FixtureFail(3011);

    DestroyWindow(parent);
    return FIXTURE_OK;
}
