#include <windows.h>
#include "../common/fixture_status.h"

#ifndef GWL_WNDPROC
#define GWL_WNDPROC (-4)
#endif

static WNDPROC g_oldProc = 0;
static DWORD g_subclassSeen = 0;

static LRESULT CALLBACK BaseProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 44) return 0x4400;
    return DefWindowProcW(hwnd, msg, wp, lp);
}

static LRESULT CALLBACK SubclassProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 45) {
        g_subclassSeen = (DWORD)wp;
        return 0x4500 + (LRESULT)wp;
    }
    return CallWindowProcW(g_oldProc, hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = BaseProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureSubclassClass";
    if (!RegisterClassW(&wc)) return FixtureFail(4401);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"subclass", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(4402);

    if (SendMessageW(hwnd, WM_USER + 44, 0, 0) != 0x4400) return FixtureFail(4403);

    g_oldProc = (WNDPROC)SetWindowLongW(hwnd, GWL_WNDPROC, (LONG)SubclassProc);
    if (!g_oldProc) return FixtureFail(4404);

    if (SendMessageW(hwnd, WM_USER + 45, 8, 0) != 0x4508 || g_subclassSeen != 8) return FixtureFail(4405);
    if (SendMessageW(hwnd, WM_USER + 44, 0, 0) != 0x4400) return FixtureFail(4406);

    SetWindowLongW(hwnd, GWL_WNDPROC, (LONG)g_oldProc);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
