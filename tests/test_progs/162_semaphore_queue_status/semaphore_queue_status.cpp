#include <windows.h>
#include "../common/fixture_status.h"

#ifndef QS_POSTMESSAGE
#define QS_POSTMESSAGE 0x0008
#endif
#ifndef QS_PAINT
#define QS_PAINT 0x0020
#endif

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 162) {
        return 0x16200 + (LRESULT)wp;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    HANDLE sem = CreateSemaphoreW(0, 1, 2, L"FixtureSemaphore162");
    if (!sem) return FixtureFail(16201);

    if (WaitForSingleObject(sem, 0) != WAIT_OBJECT_0) return FixtureFail(16202);
    if (WaitForSingleObject(sem, 0) != WAIT_TIMEOUT) return FixtureFail(16203);

    LONG previous = -1;
    if (!ReleaseSemaphore(sem, 2, &previous)) return FixtureFail(16204);
    if (previous != 0) return FixtureFail(16205);
    if (ReleaseSemaphore(sem, 1, 0)) return FixtureFail(16206);
    CloseHandle(sem);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixtureQueueStatus162";
    if (!RegisterClassW(&wc)) return FixtureFail(16207);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"queue", WS_VISIBLE, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(16208);

    DWORD status = GetQueueStatus(QS_PAINT);
    if ((status & QS_PAINT) == 0) return FixtureFail(16209);

    MSG msg;
    if (!PeekMessageW(&msg, hwnd, WM_PAINT, WM_PAINT, PM_REMOVE)) return FixtureFail(16210);
    if (msg.message != WM_PAINT || msg.hwnd != hwnd) return FixtureFail(16211);
    ValidateRect(hwnd, 0);

    status = GetQueueStatus(QS_POSTMESSAGE);
    if ((status & QS_POSTMESSAGE) != 0) return FixtureFail(16212);
    if (!PostMessageW(hwnd, WM_USER + 162, 7, 9)) return FixtureFail(16213);
    status = GetQueueStatus(QS_POSTMESSAGE);
    if ((status & QS_POSTMESSAGE) == 0) return FixtureFail(16214);

    if (!GetMessageW(&msg, 0, WM_USER + 162, WM_USER + 162)) return FixtureFail(16215);
    if (msg.hwnd != hwnd || msg.message != WM_USER + 162) return FixtureFail(16216);
    if (msg.wParam != 7 || msg.lParam != 9) return FixtureFail(16217);

    status = GetQueueStatus(QS_POSTMESSAGE);
    if ((status & QS_POSTMESSAGE) != 0) return FixtureFail(16218);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
