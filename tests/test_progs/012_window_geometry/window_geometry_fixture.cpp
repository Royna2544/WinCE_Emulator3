#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureGeometryWindow";

static LRESULT CALLBACK FixtureWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

static int CheckRect(const RECT* rect, LONG left, LONG top, LONG right, LONG bottom) {
    return rect->left == left && rect->top == top && rect->right == right && rect->bottom == bottom;
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

    HWND hwnd = CreateWindowExW(0, kClassName, L"geometry", WS_VISIBLE, 10, 20, 120, 80, 0, 0, hInstance, 0);
    if (!hwnd || !IsWindow(hwnd)) {
        return FixtureFail(2);
    }
    if (!IsWindowVisible(hwnd)) {
        return FixtureFail(3);
    }

    RECT windowRect;
    RECT clientRect;
    if (!GetWindowRect(hwnd, &windowRect)) {
        return FixtureFail(4);
    }
    if (!CheckRect(&windowRect, 10, 20, 130, 100)) {
        return FixtureFail(5);
    }
    if (!GetClientRect(hwnd, &clientRect)) {
        return FixtureFail(6);
    }
    if (!CheckRect(&clientRect, 0, 0, 120, 80)) {
        return FixtureFail(7);
    }

    if (!MoveWindow(hwnd, 30, 40, 160, 90, TRUE)) {
        return FixtureFail(8);
    }
    if (!GetWindowRect(hwnd, &windowRect) || !CheckRect(&windowRect, 30, 40, 190, 130)) {
        return FixtureFail(9);
    }

    if (!SetWindowPos(hwnd, 0, 35, 45, 170, 95, SWP_NOZORDER | SWP_NOACTIVATE)) {
        return FixtureFail(10);
    }
    if (!GetWindowRect(hwnd, &windowRect) || !CheckRect(&windowRect, 35, 45, 205, 140)) {
        return FixtureFail(11);
    }

    ShowWindow(hwnd, SW_HIDE);
    if (IsWindowVisible(hwnd)) {
        return FixtureFail(12);
    }
    ShowWindow(hwnd, SW_SHOW);
    if (!IsWindowVisible(hwnd)) {
        return FixtureFail(13);
    }

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
