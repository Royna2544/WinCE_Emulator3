#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureFocusWindow";

static LRESULT CALLBACK FixtureWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

static HWND MakeWindow(HINSTANCE instance, const wchar_t* title) {
    return CreateWindowExW(0, kClassName, title, WS_VISIBLE, 0, 0, 60, 30, 0, 0, instance, 0);
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

    HWND a = MakeWindow(hInstance, L"a");
    HWND b = MakeWindow(hInstance, L"b");
    if (!a || !b) {
        return FixtureFail(2);
    }

    if (!IsWindowEnabled(a) || !EnableWindow(a, FALSE)) {
        return FixtureFail(3);
    }
    if (IsWindowEnabled(a)) {
        return FixtureFail(4);
    }
    if (EnableWindow(a, TRUE)) {
        return FixtureFail(5);
    }
    if (!IsWindowEnabled(a)) {
        return FixtureFail(6);
    }

    SetFocus(a);
    if (GetFocus() != a) {
        return FixtureFail(7);
    }
    SetFocus(b);
    if (GetFocus() != b) {
        return FixtureFail(8);
    }

    DestroyWindow(b);
    DestroyWindow(a);
    return FIXTURE_OK;
}
