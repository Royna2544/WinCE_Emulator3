#include <windows.h>

#pragma comment(lib, "coredll.lib")

#define IDC_LBL_CURRENT  1001
#define IDC_RADIO_RNDIS  1002
#define IDC_RADIO_SERIAL 1003
#define IDC_RADIO_MSF    1004
#define IDC_RADIO_NONE   1005
#define IDC_BTN_APPLY    1006

#define REG_KEY L"Drivers\\USB\\FunctionDrivers"
#define REG_VAL L"DefaultClientDriver"

enum UsbMode { MODE_NONE = 0, MODE_RNDIS, MODE_SERIAL, MODE_MSF };

static HINSTANCE g_instance;
static HWND g_main;
static HWND g_lbl_current;
static HWND g_radios[4];   /* RNDIS, Serial, MSF, None — index matches UsbMode-1 except None=3 */
static UsbMode g_current;

static UsbMode ReadMode() {
    HKEY key;
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, REG_KEY, 0, 0, &key) != ERROR_SUCCESS)
        return MODE_NONE;
    WCHAR buf[64] = {0};
    DWORD size = sizeof(buf), type = 0;
    BOOL ok = RegQueryValueExW(key, REG_VAL, NULL, &type, (LPBYTE)buf, &size) == ERROR_SUCCESS
              && type == REG_SZ;
    RegCloseKey(key);
    if (!ok || !buf[0]) return MODE_NONE;
    if (lstrcmpiW(buf, L"RNDIS") == 0) return MODE_RNDIS;
    if (lstrcmpiW(buf, L"Serial_Class") == 0) return MODE_SERIAL;
    if (lstrcmpiW(buf, L"Mass_Storage_Class") == 0) return MODE_MSF;
    return MODE_NONE;
}

static LPCWSTR ModeFriendly(UsbMode m) {
    switch (m) {
        case MODE_RNDIS:  return L"RNDIS";
        case MODE_SERIAL: return L"Serial";
        case MODE_MSF:    return L"Mass Storage";
        default:          return L"None";
    }
}

static LPCWSTR ModeRegStr(UsbMode m) {
    switch (m) {
        case MODE_RNDIS:  return L"RNDIS";
        case MODE_SERIAL: return L"Serial_Class";
        case MODE_MSF:    return L"Mass_Storage_Class";
        default:          return NULL;
    }
}

static UsbMode RadioToMode(int idx) {
    switch (idx) {
        case 0: return MODE_RNDIS;
        case 1: return MODE_SERIAL;
        case 2: return MODE_MSF;
        default: return MODE_NONE;
    }
}

static int ModeToRadioIdx(UsbMode m) {
    switch (m) {
        case MODE_RNDIS:  return 0;
        case MODE_SERIAL: return 1;
        case MODE_MSF:    return 2;
        default:          return 3;
    }
}

static UsbMode GetSelectedMode() {
    for (int i = 0; i < 4; i++) {
        if (SendMessageW(g_radios[i], BM_GETCHECK, 0, 0) == BST_CHECKED)
            return RadioToMode(i);
    }
    return MODE_NONE;
}

static void RefreshCurrentLabel() {
    WCHAR buf[64];
    wsprintfW(buf, L"Current: %s", ModeFriendly(g_current));
    SetWindowTextW(g_lbl_current, buf);
}

static void ShowRegError(LPCWSTR op, LONG err) {
    WCHAR buf[128];
    wsprintfW(buf, L"%s failed (error %ld).", op, err);
    MessageBoxW(g_main, buf, L"USB Mode Selector", MB_OK | MB_ICONERROR);
}

static void OnApply() {
    UsbMode selected = GetSelectedMode();
    if (selected == g_current) {
        MessageBoxW(g_main, L"Not changed.", L"USB Mode Selector",
                    MB_OK | MB_ICONINFORMATION);
        return;
    }

    HKEY key;
    LONG err;

    if (selected == MODE_NONE) {
        err = RegOpenKeyExW(HKEY_LOCAL_MACHINE, REG_KEY, 0, 0, &key);
        if (err != ERROR_SUCCESS) {
            ShowRegError(L"RegOpenKeyEx", err);
            return;
        }
        err = RegDeleteValueW(key, REG_VAL);
        RegFlushKey(key);
        RegCloseKey(key);
        if (err != ERROR_SUCCESS) {
            ShowRegError(L"RegDeleteValue", err);
            return;
        }
    } else {
        err = RegOpenKeyExW(HKEY_LOCAL_MACHINE, REG_KEY, 0, 0, &key);
        if (err != ERROR_SUCCESS) {
            DWORD disp;
            err = RegCreateKeyExW(HKEY_LOCAL_MACHINE, REG_KEY, 0, NULL, 0,
                                  0, NULL, &key, &disp);
            if (err != ERROR_SUCCESS) {
                ShowRegError(L"RegCreateKeyEx", err);
                return;
            }
        }
        LPCWSTR val = ModeRegStr(selected);
        DWORD len = ((DWORD)lstrlenW(val) + 1) * sizeof(WCHAR);
        err = RegSetValueExW(key, REG_VAL, 0, REG_SZ, (const BYTE *)val, len);
        RegFlushKey(key);
        RegCloseKey(key);
        if (err != ERROR_SUCCESS) {
            ShowRegError(L"RegSetValueEx", err);
            return;
        }
    }

    g_current = selected;
    RefreshCurrentLabel();
    MessageBoxW(g_main,
                L"Done. A reboot is required for the change to take effect.",
                L"USB Mode Selector", MB_OK | MB_ICONINFORMATION);
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_COMMAND && LOWORD(wp) == IDC_BTN_APPLY && HIWORD(wp) == BN_CLICKED)
        OnApply();
    if (msg == WM_CLOSE)
        PostQuitMessage(0);
    return DefWindowProcW(hwnd, msg, wp, lp);
}

int WINAPI WinMain(HINSTANCE instance, HINSTANCE, LPWSTR, int) {
    g_instance = instance;
    g_current = ReadMode();

    WNDCLASSW wc = {0};
    wc.lpfnWndProc   = WndProc;
    wc.hInstance     = instance;
    wc.hbrBackground = (HBRUSH)(COLOR_BTNFACE + 1);
    wc.lpszClassName = L"UsbModeSelector";
    RegisterClassW(&wc);

    const int W = 220, H = 210;
    g_main = CreateWindowExW(0, L"UsbModeSelector", L"USB Mode Selector",
                             WS_VISIBLE | WS_CAPTION | WS_SYSMENU,
                             CW_USEDEFAULT, CW_USEDEFAULT, W, H,
                             NULL, NULL, instance, NULL);

    int y = 8;

    g_lbl_current = CreateWindowExW(0, L"STATIC", L"",
                                    WS_VISIBLE | WS_CHILD | SS_LEFT,
                                    8, y, W - 16, 20, g_main,
                                    (HMENU)IDC_LBL_CURRENT, instance, NULL);
    RefreshCurrentLabel();
    y += 28;

    static const WCHAR *labels[] = { L"RNDIS", L"Serial", L"Mass Storage", L"None" };
    for (int i = 0; i < 4; i++) {
        DWORD style = WS_VISIBLE | WS_CHILD | BS_AUTORADIOBUTTON | WS_TABSTOP;
        if (i == 0) style |= WS_GROUP;
        g_radios[i] = CreateWindowExW(0, L"BUTTON", labels[i], style,
                                      16, y, W - 32, 22, g_main,
                                      (HMENU)(IDC_RADIO_RNDIS + i), instance, NULL);
        y += 24;
    }
    SendMessageW(g_radios[ModeToRadioIdx(g_current)], BM_SETCHECK, BST_CHECKED, 0);

    y += 6;
    CreateWindowExW(0, L"BUTTON", L"Apply",
                    WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON | WS_TABSTOP,
                    W / 2 - 40, y, 80, 26, g_main,
                    (HMENU)IDC_BTN_APPLY, instance, NULL);

    MSG msg;
    while (GetMessageW(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    return 0;
}
