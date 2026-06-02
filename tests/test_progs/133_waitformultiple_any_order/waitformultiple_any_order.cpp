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
    s->marker = 0x085085;
    if (s->eventHandle) SetEvent(s->eventHandle);
    if (s->hwnd) PostMessageW(s->hwnd, WM_USER + 133, 133, 0);
    return 0;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 133) return 0x08500 + (LRESULT)wp;
    if (msg == WM_TIMER) { KillTimer(hwnd, 133); return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(13301);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"SchedFixtureClass133";
    RegisterClassW(&wc);

    s.hwnd = CreateWindowExW(0, wc.lpszClassName, L"scheduler fixture", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!s.hwnd) return FixtureFail(13302);

    volatile DWORD a = 0xaaaa0085;
    volatile DWORD b = 0xbbbb0085;

    DWORD tid = 0;
    HANDLE thread = CreateThread(0, 0, WorkerProc, &s, 0, &tid);
    if (!thread) return FixtureFail(13303);

    DWORD wait = WaitForSingleObject(s.eventHandle, 5000);
    if (wait != WAIT_OBJECT_0) return FixtureFail(13304);
    if (s.marker != 0x085085) return FixtureFail(13305);
    if (a != 0xaaaa0085 || b != 0xbbbb0085) return FixtureFail(13306);

    LRESULT sr = SendMessageW(s.hwnd, WM_USER + 133, 3, 0);
    if (sr != 0x08503) return FixtureFail(13307);

    PostMessageW(s.hwnd, WM_USER + 133, 4, 0);
    MSG msg;
    DWORD seen = 0;
    DWORD spins = 0;
    while (!seen && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_USER + 133) seen = 1;
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }
    if (!seen) return FixtureFail(13308);

    WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    CloseHandle(s.eventHandle);
    DestroyWindow(s.hwnd);
    return FIXTURE_OK;
}
