#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static BOOL CALLBACK DlgProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_INITDIALOG) {
        SetDlgItemTextW(hwnd, IDC_TEXT_EDIT, L"dlgitem-ok");
        wchar_t buf[64];
        ZeroMemory(buf, sizeof(buf));
        GetDlgItemTextW(hwnd, IDC_TEXT_EDIT, buf, 64);
        EndDialog(hwnd, WideEqAscii(buf, "dlgitem-ok") ? 0 : 1);
        return TRUE;
    }
    return FALSE;
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    INT_PTR r = DialogBoxParamW(h, MAKEINTRESOURCEW(IDD_TEXT_DIALOG), 0, DlgProc, 0);
    return r == 0 ? FIXTURE_OK : FixtureFail(7801);
}
