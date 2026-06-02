#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_timer = 0;
static DWORD g_paint = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_TIMER) {
        ++g_timer;
        InvalidateRect(hwnd, 0, TRUE);
        UpdateWindow(hwnd);
        if (g_timer >= 3) {
            KillTimer(hwnd, 34);
            PostQuitMessage(0);
        }
        return 0;
    }

    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);
        RECT rc;
        SetRect(&rc, 0, 0, 100, 60);
        HBRUSH brush = CreateSolidBrush(RGB(0, 80, 40 + g_paint * 30));
        FillRect(dc, &rc, brush);
        DeleteObject(brush);
        EndPaint(hwnd, &ps);
        ++g_paint;
        return 0;
    }

    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureTimerUpdateClass";
    if (!RegisterClassW(&wc)) return FixtureFail(3401);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"timer update", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(3402);

    if (!SetTimer(hwnd, 34, 10, 0)) return FixtureFail(3403);

    MSG msg;
    DWORD spins = 0;
    while (spins++ < 1000 && GetMessageW(&msg, 0, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
        if (g_timer >= 3) break;
    }

    if (g_timer < 3) return FixtureFail(3404);
    if (g_paint < 3) return FixtureFail(3405);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
