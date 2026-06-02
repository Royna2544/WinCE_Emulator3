#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_quitSeen = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 75) {
        PostQuitMessage(75);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixturePostQuitClass";
    if (!RegisterClassW(&wc)) return FixtureFail(7501);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"postquit", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(7502);

    PostMessageW(hwnd, WM_USER + 75, 0, 0);

    MSG msg;
    while (GetMessageW(&msg, 0, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    if (msg.wParam != 75) return FixtureFail(7503);
    g_quitSeen = 1;

    DestroyWindow(hwnd);
    return g_quitSeen ? FIXTURE_OK : FixtureFail(7504);
}
