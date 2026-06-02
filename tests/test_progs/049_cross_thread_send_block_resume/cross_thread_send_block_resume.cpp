#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* CLS = L"FixtureCrossThreadSendResumeClass";

struct Shared {
    HANDLE ready;
    HANDLE release;
    HWND hwnd;
    volatile DWORD enteredWndProc;
    volatile DWORD sendReturned;
};

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    Shared* s = (Shared*)GetWindowLongW(hwnd, 0);
    if (msg == WM_CREATE) {
        CREATESTRUCTW* cs = (CREATESTRUCTW*)lp;
        SetWindowLongW(hwnd, 0, (LONG)cs->lpCreateParams);
        return 0;
    }
    if (msg == WM_USER + 49) {
        s->enteredWndProc = 1;
        WaitForSingleObject(s->release, 5000);
        return 0x4900 + (LRESULT)wp;
    }
    if (msg == WM_USER + 50) {
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

static DWORD WINAPI UiThread(LPVOID p) {
    Shared* s = (Shared*)p;
    HINSTANCE h = GetModuleHandleW(0);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.cbWndExtra = sizeof(LONG);
    wc.hInstance = h;
    wc.lpszClassName = CLS;
    RegisterClassW(&wc);

    s->hwnd = CreateWindowExW(0, CLS, L"cross-thread send", WS_VISIBLE, 0, 0, 100, 60, 0, 0, h, s);
    SetEvent(s->ready);

    MSG msg;
    while (GetMessageW(&msg, 0, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    return 0;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.ready = CreateEventW(0, TRUE, FALSE, 0);
    s.release = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.ready || !s.release) return FixtureFail(4901);

    DWORD tid = 0;
    HANDLE th = CreateThread(0, 0, UiThread, &s, 0, &tid);
    if (!th) return FixtureFail(4902);
    if (WaitForSingleObject(s.ready, 5000) != WAIT_OBJECT_0 || !s.hwnd) return FixtureFail(4903);

    volatile DWORD a = 0xaaaaaaaa;
    volatile DWORD b = 0xbbbbbbbb;

    /*
       The UI thread WndProc waits on s.release. The sending thread must block
       inside SendMessageW and later resume with locals intact.
    */
    SetEvent(s.release);
    LRESULT r = SendMessageW(s.hwnd, WM_USER + 49, 7, 0);
    s.sendReturned = 1;

    if (r != 0x4907) return FixtureFail(4904);
    if (!s.enteredWndProc || !s.sendReturned) return FixtureFail(4905);
    if (a != 0xaaaaaaaa || b != 0xbbbbbbbb) return FixtureFail(4906);

    PostMessageW(s.hwnd, WM_USER + 50, 0, 0);
    WaitForSingleObject(th, 5000);

    CloseHandle(th);
    CloseHandle(s.release);
    CloseHandle(s.ready);
    return FIXTURE_OK;
}
