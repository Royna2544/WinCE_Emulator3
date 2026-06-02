#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif
#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static DWORD g_init = 0;
static DWORD g_done = 0;

static BOOL CALLBACK DlgProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_INITDIALOG) {
        g_init = 1;
        SetFocus(GetDlgItem(hwnd, IDC_EDIT_A));
        return FALSE;
    }
    if (msg == WM_COMMAND && LOWORD(wp) == IDC_BUTTON_OK) {
        g_done = 1;
        EndDialog(hwnd, 76);
        return TRUE;
    }
    if (msg == WM_CLOSE) {
        EndDialog(hwnd, 76);
        return TRUE;
    }
    return FALSE;
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    HWND dlg = CreateDialogParamW(h, MAKEINTRESOURCEW(IDD_DIALOG_KEYS), 0, DlgProc, 0);
    if (!dlg || !g_init) return FixtureFail(7601);

    MSG tabMsg;
    ZeroMemory(&tabMsg, sizeof(tabMsg));
    tabMsg.hwnd = GetDlgItem(dlg, IDC_EDIT_A);
    tabMsg.message = WM_KEYDOWN;
    tabMsg.wParam = VK_TAB;

    IsDialogMessageW(dlg, &tabMsg);

    MSG enterMsg;
    ZeroMemory(&enterMsg, sizeof(enterMsg));
    enterMsg.hwnd = GetDlgItem(dlg, IDC_BUTTON_OK);
    enterMsg.message = WM_KEYDOWN;
    enterMsg.wParam = VK_RETURN;

    SendMessageW(dlg, WM_COMMAND, IDC_BUTTON_OK, (LPARAM)GetDlgItem(dlg, IDC_BUTTON_OK));

    MSG msg;
    DWORD spins = 0;
    while (!g_done && spins++ < 50) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            if (!IsDialogMessageW(dlg, &msg)) {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Sleep(1);
    }

    if (!g_done) return FixtureFail(7602);
    return FIXTURE_OK;
}
