#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureMessageQueueWindow";
static const UINT kMsg = WM_USER + 0x21;

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

    HWND hwnd = CreateWindowExW(0, kClassName, L"messages", 0, 0, 0, 20, 20, 0, 0, hInstance, 0);
    if (!hwnd) {
        return FixtureFail(2);
    }
    if (!PostMessageW(hwnd, kMsg, 0x1234, 0x5678)) {
        return FixtureFail(3);
    }

    MSG msg;
    ZeroMemory(&msg, sizeof(msg));
    if (!PeekMessageW(&msg, hwnd, kMsg, kMsg, PM_NOREMOVE)) {
        return FixtureFail(4);
    }
    if (msg.hwnd != hwnd || msg.message != kMsg || msg.wParam != 0x1234 || msg.lParam != 0x5678) {
        return FixtureFail(5);
    }

    ZeroMemory(&msg, sizeof(msg));
    if (GetMessageW(&msg, hwnd, kMsg, kMsg) <= 0) {
        return FixtureFail(6);
    }
    if (msg.hwnd != hwnd || msg.message != kMsg || msg.wParam != 0x1234 || msg.lParam != 0x5678) {
        return FixtureFail(7);
    }

    if (PeekMessageW(&msg, hwnd, kMsg, kMsg, PM_NOREMOVE)) {
        return FixtureFail(8);
    }

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
