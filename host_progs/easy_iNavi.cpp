#include <windows.h>

#pragma comment(lib, "coredll.lib")

#define BTN_INAVI 1001
#define BTN_EXPLORER 1002

static HINSTANCE g_instance;
static HWND g_main;
static HWND g_inavi;
static HWND g_explorer;
static HANDLE g_singleton;
static HANDLE g_inavi_process;

static BOOL IsProcessStillRunning(HANDLE process) {
    return process && WaitForSingleObject(process, 0) == WAIT_TIMEOUT;
}

static BOOL LaunchPath(LPCWSTR path, HANDLE *tracked_process) {
    PROCESS_INFORMATION pi;
    ZeroMemory(&pi, sizeof(pi));
    if (tracked_process && IsProcessStillRunning(*tracked_process)) {
        WCHAR msg[180];
        wsprintfW(msg, L"Already running:\r\n%s", path);
        MessageBoxW(g_main, msg, L"easy_iNavi", MB_OK | MB_ICONINFORMATION);
        return FALSE;
    }
    if (tracked_process && *tracked_process) {
        CloseHandle(*tracked_process);
        *tracked_process = NULL;
    }
    if (!CreateProcessW(path, NULL, NULL, NULL, FALSE, 0, NULL, NULL, NULL, &pi)) {
        WCHAR msg[160];
        wsprintfW(msg, L"Could not launch:\r\n%s\r\nGetLastError=%lu", path, GetLastError());
        MessageBoxW(g_main, msg, L"easy_iNavi", MB_OK | MB_ICONERROR);
        return FALSE;
    }
    CloseHandle(pi.hThread);
    if (tracked_process) {
        *tracked_process = pi.hProcess;
    } else {
        CloseHandle(pi.hProcess);
    }
    return TRUE;
}

static void Layout(HWND hwnd) {
    RECT rc;
    GetClientRect(hwnd, &rc);
    int width = rc.right - rc.left;
    int height = rc.bottom - rc.top;
    int button_w = width * 4 / 5;
    int button_h = height / 5;
    int left = (width - button_w) / 2;
    int top = height / 4;
    MoveWindow(g_inavi, left, top, button_w, button_h, TRUE);
    MoveWindow(g_explorer, left, top + button_h + height / 10, button_w, button_h, TRUE);
}

static LRESULT CALLBACK MainProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    switch (msg) {
    case WM_CREATE:
        g_inavi = CreateWindowW(L"BUTTON", L"Launch iNavi",
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            0, 0, 0, 0, hwnd, (HMENU)BTN_INAVI, g_instance, NULL);
        g_explorer = CreateWindowW(L"BUTTON", L"Launch Explorer",
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            0, 0, 0, 0, hwnd, (HMENU)BTN_EXPLORER, g_instance, NULL);
        Layout(hwnd);
        return 0;
    case WM_SIZE:
        Layout(hwnd);
        return 0;
    case WM_COMMAND:
        if (LOWORD(wp) == BTN_INAVI) {
            LaunchPath(L"\\SDMMC Disk\\INavi\\iNavi_main.exe", &g_inavi_process);
            return 0;
        }
        if (LOWORD(wp) == BTN_EXPLORER) {
            LaunchPath(L"\\Windows\\explorer.exe", NULL);
            return 0;
        }
        break;
    case WM_DESTROY:
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

static BOOL RegisterMainClass() {
    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = MainProc;
    wc.hInstance = g_instance;
    wc.hbrBackground = (HBRUSH)GetStockObject(WHITE_BRUSH);
    wc.lpszClassName = L"EasyINaviWindow";
    return RegisterClassW(&wc) != 0;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int show_cmd) {
    MSG msg;
    g_instance = hInstance;
    g_singleton = CreateMutexW(NULL, TRUE, L"EasyINaviSingleton");
    if (g_singleton && GetLastError() == ERROR_ALREADY_EXISTS) {
        MessageBoxW(NULL, L"easy_iNavi is already running.", L"easy_iNavi",
            MB_OK | MB_ICONINFORMATION);
        CloseHandle(g_singleton);
        return 0;
    }
    if (!RegisterMainClass()) {
        if (g_singleton) {
            ReleaseMutex(g_singleton);
            CloseHandle(g_singleton);
        }
        return 1;
    }
    g_main = CreateWindowW(L"EasyINaviWindow", L"easy_iNavi",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT,
        GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN),
        NULL, NULL, hInstance, NULL);
    if (!g_main) {
        if (g_singleton) {
            ReleaseMutex(g_singleton);
            CloseHandle(g_singleton);
        }
        return 1;
    }
    ShowWindow(g_main, show_cmd);
    UpdateWindow(g_main);
    while (GetMessageW(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    if (g_inavi_process) {
        CloseHandle(g_inavi_process);
    }
    if (g_singleton) {
        ReleaseMutex(g_singleton);
        CloseHandle(g_singleton);
    }
    return (int)msg.wParam;
}
