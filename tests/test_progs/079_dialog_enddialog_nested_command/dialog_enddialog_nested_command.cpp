#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static DWORD g_order[4];
static DWORD g_count = 0;

static BOOL CALLBACK DlgProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_INITDIALOG) {
        g_order[g_count++] = 1;
        SendMessageW(hwnd, WM_COMMAND, IDC_NESTED_BUTTON, (LPARAM)GetDlgItem(hwnd, IDC_NESTED_BUTTON));
        g_order[g_count++] = 3;
        return TRUE;
    }
    if (msg == WM_COMMAND && LOWORD(wp) == IDC_NESTED_BUTTON) {
        g_order[g_count++] = 2;
        EndDialog(hwnd, 79);
        return TRUE;
    }
    return FALSE;
}

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    INT_PTR r = DialogBoxParamW(h, MAKEINTRESOURCEW(IDD_NESTED_COMMAND), 0, DlgProc, 0);
    if (r != 79) return FixtureFail(7901);
    if (g_count < 2 || g_order[0] != 1 || g_order[1] != 2) return FixtureFail(7902);
    return FIXTURE_OK;
}
