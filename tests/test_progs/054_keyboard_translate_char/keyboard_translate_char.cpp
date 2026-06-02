#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_keydown = 0;
static DWORD g_char = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_KEYDOWN) {
        g_keydown = (DWORD)wp;
        return 0;
    }
    if (msg == WM_CHAR) {
        g_char = (DWORD)wp;
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureKeyboardClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5401);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"keyboard", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(5402);

    SetFocus(hwnd);
    PostMessageW(hwnd, WM_KEYDOWN, 'A', 0);

    MSG msg;
    DWORD spins = 0;
    while ((g_keydown == 0 || g_char == 0) && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }

    if (g_keydown != 'A') return FixtureFail(5403);
    if (g_char != 'A' && g_char != 'a') return FixtureFail(5404);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
