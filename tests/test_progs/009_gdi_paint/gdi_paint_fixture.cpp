#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* kClassName = L"FixturePaintWindow";

static LRESULT CALLBACK FixtureWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_CREATE:
        InvalidateRect(hwnd, 0, TRUE);
        return 0;

    case WM_PAINT: {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);

        RECT bg;
        bg.left = 0;
        bg.top = 0;
        bg.right = 120;
        bg.bottom = 80;

        HBRUSH brush = CreateSolidBrush(RGB(32, 64, 192));
        FillRect(dc, &bg, brush);
        DeleteObject(brush);

        HBRUSH ellipseBrush = CreateSolidBrush(RGB(220, 40, 40));
        HBRUSH oldBrush = (HBRUSH)SelectObject(dc, ellipseBrush);
        Ellipse(dc, 20, 20, 100, 70);
        SelectObject(dc, oldBrush);
        DeleteObject(ellipseBrush);

        EndPaint(hwnd, &ps);
        PostQuitMessage(0);
        return 0;
    }

    case WM_DESTROY:
        PostQuitMessage(0);
        return 0;
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

    HWND hwnd = CreateWindowExW(
        0,
        kClassName,
        L"fixture paint",
        WS_VISIBLE,
        0,
        0,
        120,
        80,
        0,
        0,
        hInstance,
        0
    );

    if (!hwnd) {
        return FixtureFail(2);
    }

    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);

    MSG msg;
    while (GetMessageW(&msg, 0, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
