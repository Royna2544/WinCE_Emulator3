#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif
#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static DWORD g_initSeen = 0;
static DWORD g_commandSeen = 0;

static BOOL CALLBACK DlgProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_INITDIALOG) {
        g_initSeen = 1;
        SetDlgItemTextW(hwnd, IDC_TEST_TEXT, L"dialog-ready");
        SendMessageW(hwnd, WM_COMMAND, IDC_TEST_BUTTON, (LPARAM)GetDlgItem(hwnd, IDC_TEST_BUTTON));
        return TRUE;
    }

    if (msg == WM_COMMAND) {
        if (LOWORD(wp) == IDC_TEST_BUTTON) {
            g_commandSeen = 1;
            EndDialog(hwnd, 77);
            return TRUE;
        }
    }

    return FALSE;
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    INT_PTR result = DialogBoxParamW(h, MAKEINTRESOURCEW(IDD_TEST_DIALOG), 0, DlgProc, 0);
    if (result != 77) return FixtureFail(5101);
    if (!g_initSeen) return FixtureFail(5102);
    if (!g_commandSeen) return FixtureFail(5103);
    return FIXTURE_OK;
}
