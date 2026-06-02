#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureSendMessageWindow";
static const UINT kMsg = WM_USER + 0x42;
static DWORD g_seenCount = 0;

static LRESULT CALLBACK FixtureWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    if (msg == kMsg) {
        ++g_seenCount;
        SetWindowLongW(hwnd, GWL_USERDATA, (LONG)wParam);
        return (LRESULT)(wParam + lParam + 7);
    }
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

    HWND hwnd = CreateWindowExW(0, kClassName, L"send", 0, 0, 0, 20, 20, 0, 0, hInstance, 0);
    if (!hwnd) {
        return FixtureFail(2);
    }

    LRESULT result = SendMessageW(hwnd, kMsg, 11, 13);
    if (result != 31) {
        return FixtureFail(3);
    }
    if (g_seenCount != 1) {
        return FixtureFail(4);
    }
    if (GetWindowLongW(hwnd, GWL_USERDATA) != 11) {
        return FixtureFail(5);
    }

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
