#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static DWORD g_commandSeen = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_COMMAND) {
        if (LOWORD(wp) == IDM_TEST_ITEM) g_commandSeen = 1;
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    HMENU menu = LoadMenuW(h, MAKEINTRESOURCEW(IDM_TEST_MENU));
    if (!menu) return FixtureFail(3101);

    CheckMenuItem(menu, IDM_TEST_ITEM, MF_CHECKED);
    CheckMenuItem(menu, IDM_TEST_ITEM, MF_UNCHECKED);

    HACCEL accel = LoadAcceleratorsW(h, MAKEINTRESOURCEW(IDA_TEST_ACCEL));
    if (!accel) return FixtureFail(3102);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureMenuAccelClass";
    if (!RegisterClassW(&wc)) return FixtureFail(3103);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"menu accel", WS_VISIBLE, 0, 0, 120, 80, 0, menu, h, 0);
    if (!hwnd) return FixtureFail(3104);

    MSG msg;
    ZeroMemory(&msg, sizeof(msg));
    msg.hwnd = hwnd;
    msg.message = WM_KEYDOWN;
    msg.wParam = 'T';
    msg.lParam = 0;

    TranslateAcceleratorW(hwnd, accel, &msg);

    SendMessageW(hwnd, WM_COMMAND, IDM_TEST_ITEM, 0);
    if (!g_commandSeen) return FixtureFail(3105);

    RemoveMenu(menu, IDM_TEST_ITEM, MF_BYCOMMAND);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
