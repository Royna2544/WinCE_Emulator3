#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* CLS = L"FixtureFindWindowWideClass";
static const wchar_t* TTL = L"Fixture FindWindow Wide Title";

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 23) return 0x2300 + (LRESULT)wp;
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = CLS;
    if (!RegisterClassW(&wc)) return FixtureFail(2301);

    WNDCLASSW got;
    ZeroMemory(&got, sizeof(got));
    if (!GetClassInfoW(h, CLS, &got)) return FixtureFail(2302);

    HWND hwnd = CreateWindowExW(0, CLS, TTL, WS_VISIBLE, 1, 2, 160, 100, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(2303);

    if (FindWindowW(CLS, 0) != hwnd) return FixtureFail(2304);
    if (FindWindowW(0, TTL) != hwnd) return FixtureFail(2305);
    if (FindWindowW(CLS, TTL) != hwnd) return FixtureFail(2306);

    wchar_t title[128];
    ZeroMemory(title, sizeof(title));
    GetWindowTextW(hwnd, title, 128);
    if (!WideEqAscii(title, "Fixture FindWindow Wide Title")) return FixtureFail(2307);

    if (!RegisterWindowMessageW(L"FixtureFindWindowWideMessage")) return FixtureFail(2308);
    if (SendMessageW(hwnd, WM_USER + 23, 7, 0) != 0x2307) return FixtureFail(2309);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
