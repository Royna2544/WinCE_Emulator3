#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureZOrderWindow";

static LRESULT CALLBACK FixtureWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

static HWND MakeWindow(HINSTANCE instance, const wchar_t* title) {
    return CreateWindowExW(0, kClassName, title, WS_VISIBLE, 0, 0, 40, 40, 0, 0, instance, 0);
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
    HWND c = MakeWindow(hInstance, L"c");
    if (!a || !b || !c) {
        return FixtureFail(2);
    }

    if (GetWindow(a, GW_HWNDNEXT) != b || GetWindow(b, GW_HWNDNEXT) != c) {
        return FixtureFail(3);
    }
    if (GetWindow(c, GW_HWNDPREV) != b || GetWindow(b, GW_HWNDPREV) != a) {
        return FixtureFail(4);
    }
    if (GetWindow(a, GW_HWNDFIRST) != a || GetWindow(a, GW_HWNDLAST) != c) {
        return FixtureFail(5);
    }

    if (!SetWindowPos(c, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE)) {
        return FixtureFail(6);
    }
    if (GetWindow(a, GW_HWNDFIRST) != c) {
        return FixtureFail(7);
    }

    if (!SetWindowPos(c, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE)) {
        return FixtureFail(8);
    }
    if (GetWindow(a, GW_HWNDLAST) != c) {
        return FixtureFail(9);
    }

    DestroyWindow(c);
    DestroyWindow(b);
    DestroyWindow(a);
    return FIXTURE_OK;
}
