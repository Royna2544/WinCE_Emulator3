#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_destroySeen = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 73) {
        DestroyWindow(hwnd);
        return 0x7300;
    }
    if (msg == WM_DESTROY) {
        g_destroySeen = 1;
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureDestroyDuringSend";
    if (!RegisterClassW(&wc)) return FixtureFail(7301);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"destroy during send", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(7302);

    LRESULT r = SendMessageW(hwnd, WM_USER + 73, 0, 0);
    if (r != 0x7300) return FixtureFail(7303);
    if (!g_destroySeen) return FixtureFail(7304);

    return FIXTURE_OK;
}
