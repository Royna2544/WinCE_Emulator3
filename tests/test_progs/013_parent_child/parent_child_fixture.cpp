#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureParentChildWindow";

static LRESULT CALLBACK FixtureWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

static HWND MakeWindow(HINSTANCE instance, const wchar_t* title, DWORD style, HWND parent, int id) {
    return CreateWindowExW(0, kClassName, title, style, 0, 0, 80, 40, parent, (HMENU)id, instance, 0);
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

    HWND parentA = MakeWindow(hInstance, L"parent-a", WS_VISIBLE, 0, 0);
    HWND parentB = MakeWindow(hInstance, L"parent-b", WS_VISIBLE, 0, 0);
    if (!parentA || !parentB) {
        return FixtureFail(2);
    }

    HWND child = MakeWindow(hInstance, L"child", WS_CHILD | WS_VISIBLE, parentA, 77);
    if (!child) {
        return FixtureFail(3);
    }
    if (GetParent(child) != parentA) {
        return FixtureFail(4);
    }
    if (GetWindowLongW(child, GWL_ID) != 77) {
        return FixtureFail(5);
    }
    if (GetWindow(parentA, GW_CHILD) != child) {
        return FixtureFail(6);
    }

    if (SetParent(child, parentB) != parentA) {
        return FixtureFail(7);
    }
    if (GetParent(child) != parentB) {
        return FixtureFail(8);
    }
    if (GetWindow(parentB, GW_CHILD) != child) {
        return FixtureFail(9);
    }

    DestroyWindow(child);
    DestroyWindow(parentB);
    DestroyWindow(parentA);
    return FIXTURE_OK;
}
