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
    s->marker = 0x07e07e;
    if (s->eventHandle) SetEvent(s->eventHandle);
    if (s->hwnd) PostMessageW(s->hwnd, WM_USER + 126, 126, 0);
    return 0;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 126) return 0x07e00 + (LRESULT)wp;
    if (msg == WM_TIMER) { KillTimer(hwnd, 126); return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(12601);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"SchedFixtureClass126";
    RegisterClassW(&wc);

    s.hwnd = CreateWindowExW(0, wc.lpszClassName, L"scheduler fixture", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!s.hwnd) return FixtureFail(12602);

    volatile DWORD a = 0xaaaa007e;
    volatile DWORD b = 0xbbbb007e;

    DWORD tid = 0;
    HANDLE thread = CreateThread(0, 0, WorkerProc, &s, 0, &tid);
    if (!thread) return FixtureFail(12603);

    DWORD wait = WaitForSingleObject(s.eventHandle, 5000);
    if (wait != WAIT_OBJECT_0) return FixtureFail(12604);
    if (s.marker != 0x07e07e) return FixtureFail(12605);
    if (a != 0xaaaa007e || b != 0xbbbb007e) return FixtureFail(12606);

    LRESULT sr = SendMessageW(s.hwnd, WM_USER + 126, 3, 0);
    if (sr != 0x07e03) return FixtureFail(12607);

    PostMessageW(s.hwnd, WM_USER + 126, 4, 0);
    MSG msg;
    DWORD seen = 0;
    DWORD spins = 0;
    while (!seen && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_USER + 126) seen = 1;
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }
    if (!seen) return FixtureFail(12608);

    WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    CloseHandle(s.eventHandle);
    DestroyWindow(s.hwnd);
    return FIXTURE_OK;
}
