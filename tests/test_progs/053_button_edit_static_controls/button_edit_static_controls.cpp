#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_command = 0;

static LRESULT CALLBACK ParentProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_COMMAND) {
        if (LOWORD(wp) == 5302) g_command = 1;
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = ParentProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureControlsParentClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5301);

    HWND parent = CreateWindowExW(0, wc.lpszClassName, L"controls", WS_VISIBLE, 0, 0, 200, 120, 0, 0, h, 0);
    if (!parent) return FixtureFail(5302);

    HWND edit = CreateWindowExW(0, L"EDIT", L"", WS_CHILD | WS_VISIBLE | WS_BORDER, 5, 5, 120, 20, parent, (HMENU)5301, h, 0);
    HWND button = CreateWindowExW(0, L"BUTTON", L"Press", WS_CHILD | WS_VISIBLE, 5, 35, 80, 20, parent, (HMENU)5302, h, 0);
    HWND stat = CreateWindowExW(0, L"STATIC", L"StaticText", WS_CHILD | WS_VISIBLE, 5, 65, 100, 20, parent, (HMENU)5303, h, 0);

    if (!edit || !button || !stat) return FixtureFail(5303);

    SetWindowTextW(edit, L"edit-ok");

    wchar_t text[64];
    ZeroMemory(text, sizeof(text));
    GetWindowTextW(edit, text, 64);
    if (!WideEqAscii(text, "edit-ok")) return FixtureFail(5304);

    if (GetDlgItem(parent, 5302) != button) return FixtureFail(5305);

    SendMessageW(parent, WM_COMMAND, 5302, (LPARAM)button);
    if (!g_command) return FixtureFail(5306);

    DestroyWindow(parent);
    return FIXTURE_OK;
}
