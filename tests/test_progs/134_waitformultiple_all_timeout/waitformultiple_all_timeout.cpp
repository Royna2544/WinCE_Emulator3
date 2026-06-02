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
    s->marker = 0x086086;
    if (s->eventHandle) SetEvent(s->eventHandle);
    if (s->hwnd) PostMessageW(s->hwnd, WM_USER + 134, 134, 0);
    return 0;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 134) return 0x08600 + (LRESULT)wp;
    if (msg == WM_TIMER) { KillTimer(hwnd, 134); return 0; }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(13401);
    HANDLE neverReady = CreateEventW(0, TRUE, FALSE, 0);
    if (!neverReady) return FixtureFail(13409);

    HANDLE waitAllHandles[2] = { s.eventHandle, neverReady };
    // CE6 NKWaitForMultipleObjects rejects fWaitAll == TRUE with
    // WAIT_FAILED/ERROR_INVALID_PARAMETER instead of timing out.
    SetLastError(0);
    DWORD allWait = WaitForMultipleObjects(2, waitAllHandles, TRUE, 1);
    if (allWait != WAIT_FAILED) return FixtureFail(13410);
    if (GetLastError() != ERROR_INVALID_PARAMETER) return FixtureFail(13411);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"SchedFixtureClass134";
    RegisterClassW(&wc);

    s.hwnd = CreateWindowExW(0, wc.lpszClassName, L"scheduler fixture", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, 0);
    if (!s.hwnd) return FixtureFail(13402);

    volatile DWORD a = 0xaaaa0086;
    volatile DWORD b = 0xbbbb0086;

    DWORD tid = 0;
    HANDLE thread = CreateThread(0, 0, WorkerProc, &s, 0, &tid);
    if (!thread) return FixtureFail(13403);

    DWORD wait = WaitForSingleObject(s.eventHandle, 5000);
    if (wait != WAIT_OBJECT_0) return FixtureFail(13404);
    if (s.marker != 0x086086) return FixtureFail(13405);
    if (a != 0xaaaa0086 || b != 0xbbbb0086) return FixtureFail(13406);

    LRESULT sr = SendMessageW(s.hwnd, WM_USER + 134, 3, 0);
    if (sr != 0x08603) return FixtureFail(13407);

    PostMessageW(s.hwnd, WM_USER + 134, 4, 0);
    MSG msg;
    DWORD seen = 0;
    DWORD spins = 0;
    while (!seen && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_USER + 134) seen = 1;
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }
    if (!seen) return FixtureFail(13408);

    WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    CloseHandle(neverReady);
    CloseHandle(s.eventHandle);
    DestroyWindow(s.hwnd);
    return FIXTURE_OK;
}
