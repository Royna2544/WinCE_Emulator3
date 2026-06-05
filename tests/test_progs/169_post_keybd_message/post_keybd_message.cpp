#include <windows.h>
#include <keybd.h>
#include "../common/fixture_status.h"

static DWORD g_keydown = 0;
static DWORD g_char = 0;
static DWORD g_keyup = 0;
static DWORD g_keyboard_source_count = 0;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_KEYDOWN) {
        g_keydown = (DWORD)wp;
        if (GetMessageSource() == MSGSRC_HARDWARE_KEYBOARD) {
            ++g_keyboard_source_count;
        }
        return 0;
    }
    if (msg == WM_CHAR) {
        g_char = (DWORD)wp;
        if (GetMessageSource() == MSGSRC_HARDWARE_KEYBOARD) {
            ++g_keyboard_source_count;
        }
        return 0;
    }
    if (msg == WM_KEYUP) {
        g_keyup = (DWORD)wp;
        if (GetMessageSource() == MSGSRC_HARDWARE_KEYBOARD) {
            ++g_keyboard_source_count;
        }
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = WndProc;
    wc.hInstance = h;
    wc.lpszClassName = L"FixturePostKeybdClass";
    if (!RegisterClassW(&wc)) return FixtureFail(16901);

    HWND hwnd = CreateWindowExW(0, wc.lpszClassName, L"postkeybd", 0, 0, 0, 80, 40, 0, 0, h, 0);
    if (!hwnd) return FixtureFail(16902);
    SetFocus(hwnd);

    if (!PostKeybdMessage(hwnd, 'Z', KeyStateDownFlag, 0, 0, 0)) return FixtureFail(16903);
    if (!PostKeybdMessage(hwnd, 'Z', KeyStatePrevDownFlag, 0, 0, 0)) return FixtureFail(16904);

    MSG msg;
    DWORD spins = 0;
    while ((g_keydown == 0 || g_char == 0 || g_keyup == 0) && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }

    if (g_keydown != 'Z') return FixtureFail(16905);
    if (g_char != 'Z') return FixtureFail(16906);
    if (g_keyup != 'Z') return FixtureFail(16907);
    if (GetKeyState('Z') < 0) return FixtureFail(16908);
    if (g_keyboard_source_count < 2) return FixtureFail(16909);

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}
