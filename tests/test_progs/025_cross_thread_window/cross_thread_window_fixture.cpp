#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* CLS = L"FixtureCrossThreadWindowClass";
static const wchar_t* TTL = L"Fixture Cross Thread Window";

struct Shared {
    HANDLE ready;
    HWND hwnd;
    DWORD gotSend;
    DWORD gotPost;
    DWORD exitSeen;
};

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    Shared* s = (Shared*)GetWindowLongW(hwnd, 0);
    if (msg == WM_CREATE) {
        CREATESTRUCTW* cs = (CREATESTRUCTW*)lp;
        SetWindowLongW(hwnd, 0, (LONG)cs->lpCreateParams);
        return 0;
    }
    if (msg == WM_USER + 25) { if (s) s->gotSend = (DWORD)wp; return 0x2500 + (LRESULT)wp; }
    if (msg == WM_USER + 26) { if (s) s->gotPost = (DWORD)wp; return 0; }
    if (msg == WM_USER + 27) { if (s) s->exitSeen = 1; PostQuitMessage(0); return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

static DWORD WINAPI ThreadProc(LPVOID p) {
    Shared* s = (Shared*)p;
    HINSTANCE h = GetModuleHandleW(0);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.cbWndExtra = sizeof(LONG);
    wc.hInstance = h;
    wc.lpszClassName = CLS;
    RegisterClassW(&wc);

    s->hwnd = CreateWindowExW(0, CLS, TTL, WS_VISIBLE, 0, 0, 140, 90, 0, 0, h, s);
    SetEvent(s->ready);
    if (!s->hwnd) return 1;

    MSG msg;
    while (GetMessageW(&msg, 0, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    DestroyWindow(s->hwnd);
    return 0;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.ready = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.ready) return FixtureFail(2501);

    DWORD tid = 0;
    HANDLE th = CreateThread(0, 0, ThreadProc, &s, 0, &tid);
    if (!th) return FixtureFail(2502);
    if (WaitForSingleObject(s.ready, 5000) != WAIT_OBJECT_0 || !s.hwnd) return FixtureFail(2503);

    if (FindWindowW(CLS, TTL) != s.hwnd) return FixtureFail(2504);
    if (SendMessageW(s.hwnd, WM_USER + 25, 6, 0) != 0x2506 || s.gotSend != 6) return FixtureFail(2505);
    if (!PostMessageW(s.hwnd, WM_USER + 26, 9, 0)) return FixtureFail(2506);

    DWORD spins = 0;
    while (s.gotPost != 9 && spins++ < 200) Sleep(5);
    if (s.gotPost != 9) return FixtureFail(2507);

    PostMessageW(s.hwnd, WM_USER + 27, 0, 0);
    if (WaitForSingleObject(th, 5000) != WAIT_OBJECT_0 || !s.exitSeen) return FixtureFail(2508);

    CloseHandle(th);
    CloseHandle(s.ready);
    return FIXTURE_OK;
}
