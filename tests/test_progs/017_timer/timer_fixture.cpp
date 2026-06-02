#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixtureTimerWindow";

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

    HWND hwnd = CreateWindowExW(0, kClassName, L"timer", 0, 0, 0, 20, 20, 0, 0, hInstance, 0);
    if (!hwnd) {
        return FixtureFail(2);
    }

    UINT_PTR timerId = SetTimer(hwnd, 42, 10, 0);
    if (timerId == 0) {
        return FixtureFail(3);
    }

    MSG msg;
    DWORD deadline = GetTickCount() + 1000;
    do {
        if (PeekMessageW(&msg, hwnd, WM_TIMER, WM_TIMER, PM_REMOVE)) {
            if (msg.hwnd != hwnd || msg.message != WM_TIMER || msg.wParam != timerId) {
                return FixtureFail(4);
            }
            if (!KillTimer(hwnd, timerId)) {
                return FixtureFail(5);
            }
            DestroyWindow(hwnd);
            return FIXTURE_OK;
        }
        Sleep(1);
    } while (GetTickCount() < deadline);

    return FixtureFail(6);
}
