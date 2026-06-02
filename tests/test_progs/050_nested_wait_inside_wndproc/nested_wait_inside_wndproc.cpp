#include <windows.h>
#include "../common/fixture_status.h"

struct State {
    HANDLE eventHandle;
    volatile DWORD workerSet;
    volatile DWORD wndWaitOk;
};

static DWORD WINAPI Worker(LPVOID p) {
    State* s = (State*)p;
    Sleep(10);
    s->workerSet = 0x5050;
    SetEvent(s->eventHandle);
    return 0;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    State* s = (State*)GetWindowLongW(hwnd, 0);
    if (msg == WM_CREATE) {
        CREATESTRUCTW* cs = (CREATESTRUCTW*)lp;
        SetWindowLongW(hwnd, 0, (LONG)cs->lpCreateParams);
        return 0;
    }
    if (msg == WM_USER + 50) {
        DWORD tid = 0;
        HANDLE th = CreateThread(0, 0, Worker, s, 0, &tid);
        if (!th) return 0x5001;
        DWORD wait = WaitForSingleObject(s->eventHandle, 5000);
        WaitForSingleObject(th, 5000);
        CloseHandle(th);
        if (wait == WAIT_OBJECT_0 && s->workerSet == 0x5050) {
            s->wndWaitOk = 1;
            return 0x5050;
        }
        return 0x5002;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    State s;
    ZeroMemory(&s, sizeof(s));
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(5001);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.cbWndExtra = sizeof(LONG);
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureNestedWaitWndProcClass";
    if (!RegisterClassW(&wc)) return FixtureFail(5002);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"nested wait", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, &s);
    if (!hwnd) return FixtureFail(5003);

    LRESULT r = SendMessageW(hwnd, WM_USER + 50, 0, 0);
    if (r != 0x5050 || !s.wndWaitOk) return FixtureFail(5004);

    CloseHandle(s.eventHandle);
    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
