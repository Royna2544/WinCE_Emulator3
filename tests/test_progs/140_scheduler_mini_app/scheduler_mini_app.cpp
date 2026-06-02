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
    s->marker = 0x08c08c;
    if (s->eventHandle) SetEvent(s->eventHandle);
    if (s->hwnd) PostMessageW(s->hwnd, WM_USER + 140, 140, 0);
    return 0;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 140) return 0x08c00 + (LRESULT)wp;
    if (msg == WM_TIMER) { KillTimer(hwnd, 140); return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(14001);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"SchedFixtureClass140";
    RegisterClassW(&wc);

    s.hwnd = CreateWindowExW(0, wc.lpszClassName, L"scheduler fixture", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!s.hwnd) return FixtureFail(14002);

    volatile DWORD a = 0xaaaa008c;
    volatile DWORD b = 0xbbbb008c;

    DWORD tid = 0;
    HANDLE thread = CreateThread(0, 0, WorkerProc, &s, 0, &tid);
    if (!thread) return FixtureFail(14003);

    DWORD wait = WaitForSingleObject(s.eventHandle, 5000);
    if (wait != WAIT_OBJECT_0) return FixtureFail(14004);
    if (s.marker != 0x08c08c) return FixtureFail(14005);
    if (a != 0xaaaa008c || b != 0xbbbb008c) return FixtureFail(14006);

    LRESULT sr = SendMessageW(s.hwnd, WM_USER + 140, 3, 0);
    if (sr != 0x08c03) return FixtureFail(14007);

    PostMessageW(s.hwnd, WM_USER + 140, 4, 0);
    MSG msg;
    DWORD seen = 0;
    DWORD spins = 0;
    while (!seen && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_USER + 140) seen = 1;
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }
    if (!seen) return FixtureFail(14008);

    WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    CloseHandle(s.eventHandle);
    DestroyWindow(s.hwnd);
    return FIXTURE_OK;
}
