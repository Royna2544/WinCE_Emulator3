#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif
#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static BOOL CALLBACK DlgProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_INITDIALOG) {
        CheckDlgButton(hwnd, IDC_CHECK_A, BST_CHECKED);
        CheckRadioButton(hwnd, IDC_RADIO_A, IDC_RADIO_B, IDC_RADIO_B);
        SendMessageW(hwnd, WM_COMMAND, IDC_DONE, (LPARAM)GetDlgItem(hwnd, IDC_DONE));
        return TRUE;
    }
    if (msg == WM_COMMAND && LOWORD(wp) == IDC_DONE) {
        if (IsDlgButtonChecked(hwnd, IDC_CHECK_A) != BST_CHECKED) EndDialog(hwnd, 1);
        else if (IsDlgButtonChecked(hwnd, IDC_RADIO_B) != BST_CHECKED) EndDialog(hwnd, 2);
        else EndDialog(hwnd, 0);
        return TRUE;
    }
    return FALSE;
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    INT_PTR r = DialogBoxParamW(h, MAKEINTRESOURCEW(IDD_RADIO_DIALOG), 0, DlgProc, 0);
    return r == 0 ? FIXTURE_OK : FixtureFail(7701 + (DWORD)r);
}
