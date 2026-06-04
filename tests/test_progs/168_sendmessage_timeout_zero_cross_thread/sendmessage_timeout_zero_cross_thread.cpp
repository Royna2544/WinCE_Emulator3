#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* CLS = L"FixtureSendTimeoutZeroCrossThread";

struct Shared {
    HANDLE ready;
    HANDLE pump;
    HWND hwnd;
    volatile DWORD gotSend;
    volatile DWORD exitSeen;
};

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    Shared* s = (Shared*)GetWindowLongW(hwnd, 0);
    if (msg == WM_CREATE) {
        CREATESTRUCTW* cs = (CREATESTRUCTW*)lp;
        SetWindowLongW(hwnd, 0, (LONG)cs->lpCreateParams);
        return 0;
    }
    if (msg == WM_USER + 168) {
        if (s) s->gotSend = (DWORD)wp;
        return 0x16800 + (LRESULT)wp;
    }
    if (msg == WM_USER + 169) {
        if (s) s->exitSeen = 1;
        PostQuitMessage(0);
        return 0;
    }
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

    s->hwnd = CreateWindowExW(0, CLS, L"timeout", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, s);
    SetEvent(s->ready);
    if (!s->hwnd) return 1;
    if (WaitForSingleObject(s->pump, 5000) != WAIT_OBJECT_0) return 2;

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
    s.pump = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.ready || !s.pump) return FixtureFail(16801);

    DWORD tid = 0;
    HANDLE th = CreateThread(0, 0, ThreadProc, &s, 0, &tid);
    if (!th) return FixtureFail(16802);
    if (WaitForSingleObject(s.ready, 5000) != WAIT_OBJECT_0 || !s.hwnd) return FixtureFail(16803);

    DWORD_PTR result = 0xfeedcafe;
    LRESULT ret = SendMessageTimeout(s.hwnd, WM_USER + 168, 0x68, 0, SMTO_NORMAL, 0, &result);
    if (ret != 0) return FixtureFail(16804);
    if (result != 0xfeedcafe) return FixtureFail(16805);
    if (s.gotSend != 0) return FixtureFail(16806);

    SetEvent(s.pump);
    if (!PostMessageW(s.hwnd, WM_USER + 169, 0, 0)) return FixtureFail(16807);
    if (WaitForSingleObject(th, 5000) != WAIT_OBJECT_0 || !s.exitSeen) return FixtureFail(16808);
    if (s.gotSend != 0) return FixtureFail(16809);

    CloseHandle(th);
    CloseHandle(s.ready);
    CloseHandle(s.pump);
    return FIXTURE_OK;
}
