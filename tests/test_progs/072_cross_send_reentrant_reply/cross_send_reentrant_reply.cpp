#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* CLASS_A = L"FixtureReentrantA";
static const wchar_t* CLASS_B = L"FixtureReentrantB";

struct Shared {
    HWND a;
    HWND b;
    DWORD order[8];
    DWORD count;
};

static Shared* g_state = 0;

static void Mark(DWORD v) {
    if (g_state && g_state->count < 8) g_state->order[g_state->count++] = v;
}

static LRESULT CALLBACK ProcA(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 72) {
        Mark(1);
        SendMessageW(g_state->b, WM_USER + 73, 0, 0);
        Mark(3);
        return 0x7200;
    }
    if (msg == WM_USER + 74) {
        Mark(2);
        return 0x7400;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

static LRESULT CALLBACK ProcB(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 73) {
        SendMessageW(g_state->a, WM_USER + 74, 0, 0);
        return 0x7300;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    g_state = &s;

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.hInstance = h;

    wc.lpfnWndProc = ProcA;
    wc.lpszClassName = CLASS_A;
    if (!RegisterClassW(&wc)) return FixtureFail(7201);

    wc.lpfnWndProc = ProcB;
    wc.lpszClassName = CLASS_B;
    if (!RegisterClassW(&wc)) return FixtureFail(7202);

    s.a = CreateWindowExW(0, CLASS_A, L"A", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    s.b = CreateWindowExW(0, CLASS_B, L"B", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!s.a || !s.b) return FixtureFail(7203);

    if (SendMessageW(s.a, WM_USER + 72, 0, 0) != 0x7200) return FixtureFail(7204);

    if (s.count != 3 || s.order[0] != 1 || s.order[1] != 2 || s.order[2] != 3) return FixtureFail(7205);

    DestroyWindow(s.b);
    DestroyWindow(s.a);
    return FIXTURE_OK;
}
