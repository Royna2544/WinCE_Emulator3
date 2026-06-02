#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static DWORD g_init = 0;
static DWORD g_done = 0;

static BOOL CALLBACK DlgProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_INITDIALOG) {
        g_init = 1;
        SetDlgItemTextW(hwnd, IDC_MODELESS_EDIT, L"modeless");
        return TRUE;
    }

    if (msg == WM_COMMAND && LOWORD(wp) == IDC_MODELESS_BUTTON) {
        g_done = 1;
        DestroyWindow(hwnd);
        PostQuitMessage(0);
        return TRUE;
    }

    return FALSE;
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    HWND dlg = CreateDialogParamW(h, MAKEINTRESOURCEW(IDD_MODELESS_DIALOG), 0, DlgProc, 0);
    if (!dlg) return FixtureFail(5201);
    if (!g_init) return FixtureFail(5202);

    PostMessageW(dlg, WM_COMMAND, IDC_MODELESS_BUTTON, (LPARAM)GetDlgItem(dlg, IDC_MODELESS_BUTTON));

    MSG msg;
    DWORD spins = 0;
    while (!g_done && spins++ < 100) {
        if (GetMessageW(&msg, 0, 0, 0) <= 0) break;
        if (!IsDialogMessageW(dlg, &msg)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    if (!g_done) return FixtureFail(5203);
    return FIXTURE_OK;
}
