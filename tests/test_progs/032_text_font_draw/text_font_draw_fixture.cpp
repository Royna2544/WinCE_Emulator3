#include <windows.h>
#include "../common/fixture_status.h"

static DWORD g_painted = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        LOGFONTW lf;
        ZeroMemory(&lf, sizeof(lf));
        lf.lfHeight = 14;
        lf.lfWeight = FW_NORMAL;
        lstrcpyW(lf.lfFaceName, L"Tahoma");

        HFONT font = CreateFontIndirectW(&lf);
        if (font) {
            HFONT old = (HFONT)SelectObject(dc, font);
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, RGB(255, 255, 255));
            SetTextAlign(dc, TA_LEFT | TA_TOP);

            RECT rc;
            SetRect(&rc, 0, 0, 160, 80);
            HBRUSH bg = CreateSolidBrush(RGB(0, 0, 80));
            FillRect(dc, &rc, bg);
            DeleteObject(bg);

            DrawTextW(dc, L"DrawTextW fixture", -1, &rc, DT_LEFT | DT_TOP);
            ExtTextOutW(dc, 2, 30, 0, 0, L"ExtTextOutW", 11, 0);

            SelectObject(dc, old);
            DeleteObject(font);
            g_painted = 1;
        }

        EndPaint(hwnd, &ps);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureTextFontClass";
    if (!RegisterClassW(&wc)) return FixtureFail(3201);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"text font", WS_VISIBLE, 0, 0, 160, 80, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(3202);

    InvalidateRect(hwnd, 0, TRUE);
    UpdateWindow(hwnd);

    MSG msg;
    DWORD spins = 0;
    while (!g_painted && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }

    if (!g_painted) return FixtureFail(3203);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
