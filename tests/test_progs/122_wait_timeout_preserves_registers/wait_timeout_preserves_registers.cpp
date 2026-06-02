#include <windows.h>
#include "../common/fixture_status.h"

struct Shared {
    HANDLE eventHandle;
    volatile DWORD marker;
    HWND hwnd;
};

static DWORD WINAPI WorkerProc(LPVOID p) {
    Shared* s = (Shared*)p;
    Sleep(10);
    s->marker = 0x07a07a;
    if (s->eventHandle) SetEvent(s->eventHandle);
    if (s->hwnd) PostMessageW(s->hwnd, WM_USER + 122, 122, 0);
    return 0;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 122) return 0x07a00 + (LRESULT)wp;
    if (msg == WM_TIMER) { KillTimer(hwnd, 122); return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(12201);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"SchedFixtureClass122";
    RegisterClassW(&wc);

    s.hwnd = CreateWindowExW(0, wc.lpszClassName, L"scheduler fixture", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!s.hwnd) return FixtureFail(12202);

    volatile DWORD a = 0xaaaa007a;
    volatile DWORD b = 0xbbbb007a;

    DWORD tid = 0;
    HANDLE thread = CreateThread(0, 0, WorkerProc, &s, 0, &tid);
    if (!thread) return FixtureFail(12203);

    DWORD wait = WaitForSingleObject(s.eventHandle, 5000);
    if (wait != WAIT_OBJECT_0) return FixtureFail(12204);
    if (s.marker != 0x07a07a) return FixtureFail(12205);
    if (a != 0xaaaa007a || b != 0xbbbb007a) return FixtureFail(12206);

    LRESULT sr = SendMessageW(s.hwnd, WM_USER + 122, 3, 0);
    if (sr != 0x07a03) return FixtureFail(12207);

    PostMessageW(s.hwnd, WM_USER + 122, 4, 0);
    MSG msg;
    DWORD seen = 0;
    DWORD spins = 0;
    while (!seen && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_USER + 122) seen = 1;
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }
    if (!seen) return FixtureFail(12208);

    WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    CloseHandle(s.eventHandle);
    DestroyWindow(s.hwnd);
    return FIXTURE_OK;
}
