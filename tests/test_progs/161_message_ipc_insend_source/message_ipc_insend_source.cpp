#include <windows.h>
#include "../common/fixture_status.h"

static volatile DWORD g_sendState = 0;
static volatile DWORD g_postState = 0;
static volatile DWORD g_postSource = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_USER + 161) {
        g_sendState = InSendMessage() ? 1 : 0xdead;
        return 0x16100 + (LRESULT)wp;
    }
    if (msg == WM_USER + 162) {
        g_postState = InSendMessage() ? 0xdead : 1;
        g_postSource = GetMessageSource();
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    if (InSendMessage()) return FixtureFail(16101);

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"MessageIpcFixture161";
    if (!RegisterClassW(&wc)) return FixtureFail(16102);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"ipc", WS_VISIBLE, 0, 0, 120, 80, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(16103);

    DWORD_PTR timeoutResult = 0;
    LRESULT timeoutRet = SendMessageTimeout(hwnd, WM_USER + 161, 7, 0, SMTO_NORMAL, 1000, &timeoutResult);
    if (timeoutRet != 0x16107) return FixtureFail(16104);
    if (timeoutResult != 0x16107) return FixtureFail(16105);
    if (g_sendState != 1) return FixtureFail(16106);
    if (InSendMessage()) return FixtureFail(16107);

    if (!PostMessageW(hwnd, WM_USER + 162, 0, 0)) return FixtureFail(16108);
    MSG msg;
    if (!GetMessageW(&msg, 0, 0, 0)) return FixtureFail(16109);
    if (msg.message != WM_USER + 162) return FixtureFail(16110);
    DispatchMessageW(&msg);
    if (g_postState != 1) return FixtureFail(16111);
    if (g_postSource != MSGSRC_SOFTWARE_POST) return FixtureFail(16112);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
