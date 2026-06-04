#include "../fixture_common.h"

static LRESULT CALLBACK MsgWaitWakeWndProc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam)
{
    return DefWindowProcW(hwnd, msg, wparam, lparam);
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int)
{
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = MsgWaitWakeWndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = L"FixtureMsgWaitWakeClass";
    if (!RegisterClassW(&wc)) return FixtureFail(16601);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"msgwait wake",
                                WS_VISIBLE, 0, 0, 80, 40, 0, 0, hInstance, 0);
    if (!hwnd) return FixtureFail(16602);

    if (!PostThreadMessageW(GetCurrentThreadId(), WM_USER + 166, 0x66, 0x99))
        return FixtureFail(16603);
    DWORD wait = MsgWaitForMultipleObjectsEx(0, 0, 1000, QS_POSTMESSAGE, MWMO_INPUTAVAILABLE);
    if (wait != WAIT_OBJECT_0) return FixtureFail(16604);

    MSG msg;
    if (!PeekMessageW(&msg, 0, WM_USER + 166, WM_USER + 166, PM_REMOVE))
        return FixtureFail(16605);
    if (msg.message != WM_USER + 166 || msg.wParam != 0x66 || msg.lParam != 0x99)
        return FixtureFail(16606);

    UINT timer = SetTimer(hwnd, 166, 1, 0);
    if (!timer) return FixtureFail(16607);
    wait = MsgWaitForMultipleObjectsEx(0, 0, 1000, QS_TIMER, MWMO_INPUTAVAILABLE);
    if (wait != WAIT_OBJECT_0) return FixtureFail(16608);
    if (!PeekMessageW(&msg, hwnd, WM_TIMER, WM_TIMER, PM_REMOVE))
        return FixtureFail(16609);
    if (msg.message != WM_TIMER || msg.wParam != timer) return FixtureFail(16610);
    KillTimer(hwnd, timer);

    DestroyWindow(hwnd);
    return FixturePass();
}
