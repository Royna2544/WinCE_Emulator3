#include <windows.h>

#pragma comment(lib, "coredll.lib")

#define BTN_INAVI 1001
#define BTN_EXPLORER 1002
#define EDIT_DUMP 1003
#define MAX_DUMP_TEXT 8192
#define MAX_PATH_CE 260

static HINSTANCE g_instance;
static HWND g_main;
static HWND g_inavi;
static HWND g_explorer;
static HWND g_dump;
static HANDLE g_singleton;
static HANDLE g_inavi_process;

typedef DWORD (WINAPI *PFN_GetCurrentDirectoryW)(DWORD, LPWSTR);
typedef LPWCH (WINAPI *PFN_GetEnvironmentStringsW)(VOID);
typedef BOOL (WINAPI *PFN_FreeEnvironmentStringsW)(LPWCH);

static FARPROC ResolveCoredllProc(LPCWSTR proc_name) {
    HMODULE module = LoadLibraryW(L"coredll.dll");
    if (!module) {
        return NULL;
    }
    return GetProcAddress(module, proc_name);
}

static void CopyString(WCHAR *dst, int dst_chars, LPCWSTR src) {
    int i = 0;
    if (!dst || dst_chars <= 0) {
        return;
    }
    if (src) {
        for (; i < dst_chars - 1 && src[i]; ++i) {
            dst[i] = src[i];
        }
    }
    dst[i] = 0;
}

static void CatString(WCHAR *dst, int dst_chars, LPCWSTR src) {
    int len = lstrlenW(dst);
    if (len < dst_chars) {
        CopyString(dst + len, dst_chars - len, src);
    }
}

static void AppendPythonString(WCHAR *dst, int dst_chars, LPCWSTR src) {
    CatString(dst, dst_chars, L"'");
    if (src) {
        for (int i = 0; src[i]; ++i) {
            WCHAR ch[2];
            ch[0] = src[i];
            ch[1] = 0;
            if (src[i] == L'\\') {
                CatString(dst, dst_chars, L"\\\\");
            } else if (src[i] == L'\'') {
                CatString(dst, dst_chars, L"\\'");
            } else if (src[i] == L'\r') {
                CatString(dst, dst_chars, L"\\r");
            } else if (src[i] == L'\n') {
                CatString(dst, dst_chars, L"\\n");
            } else {
                CatString(dst, dst_chars, ch);
            }
        }
    }
    CatString(dst, dst_chars, L"'");
}

static void AppendLine(WCHAR *dst, int dst_chars, LPCWSTR line) {
    CatString(dst, dst_chars, line);
    CatString(dst, dst_chars, L"\r\n");
}

static BOOL NextCommandLineArg(LPCWSTR cmd, int *pos, WCHAR *out, int out_chars) {
    int i = *pos;
    int out_i = 0;
    BOOL quoted = FALSE;
    if (!cmd || !out || out_chars <= 0) {
        return FALSE;
    }
    while (cmd[i] == L' ' || cmd[i] == L'\t') {
        ++i;
    }
    if (!cmd[i]) {
        out[0] = 0;
        *pos = i;
        return FALSE;
    }
    while (cmd[i]) {
        WCHAR ch = cmd[i++];
        if (ch == L'"') {
            quoted = !quoted;
            continue;
        }
        if (!quoted && (ch == L' ' || ch == L'\t')) {
            break;
        }
        if (out_i < out_chars - 1) {
            out[out_i++] = ch;
        }
    }
    out[out_i] = 0;
    *pos = i;
    return TRUE;
}

static void AppendCommandLineArgs(WCHAR *dst, int dst_chars) {
    LPCWSTR cmd = GetCommandLineW();
    WCHAR arg[MAX_PATH_CE];
    int pos = 0;
    BOOL first = TRUE;
    CatString(dst, dst_chars, L"process args = [");
    while (NextCommandLineArg(cmd, &pos, arg, MAX_PATH_CE)) {
        if (!first) {
            CatString(dst, dst_chars, L", ");
        }
        AppendPythonString(dst, dst_chars, arg);
        first = FALSE;
    }
    AppendLine(dst, dst_chars, L"]");
}

static void AppendCurrentDirectory(WCHAR *dst, int dst_chars) {
    WCHAR cwd[MAX_PATH_CE];
    PFN_GetCurrentDirectoryW proc =
        (PFN_GetCurrentDirectoryW)ResolveCoredllProc(L"GetCurrentDirectoryW");
    CatString(dst, dst_chars, L"cwd = ");
    if (proc && proc(MAX_PATH_CE, cwd) != 0) {
        AppendPythonString(dst, dst_chars, cwd);
    } else {
        AppendPythonString(dst, dst_chars, L"<unavailable>");
    }
    AppendLine(dst, dst_chars, L"");
}

static void AppendEnvironment(WCHAR *dst, int dst_chars) {
    PFN_GetEnvironmentStringsW get_env =
        (PFN_GetEnvironmentStringsW)ResolveCoredllProc(L"GetEnvironmentStringsW");
    PFN_FreeEnvironmentStringsW free_env =
        (PFN_FreeEnvironmentStringsW)ResolveCoredllProc(L"FreeEnvironmentStringsW");
    LPWCH block;
    BOOL first = TRUE;
    CatString(dst, dst_chars, L"process env = {");
    if (!get_env) {
        AppendLine(dst, dst_chars, L"}");
        return;
    }
    block = get_env();
    if (!block) {
        AppendLine(dst, dst_chars, L"}");
        return;
    }
    for (LPWCH entry = block; *entry; entry += lstrlenW(entry) + 1) {
        WCHAR *equals = entry;
        while (*equals && *equals != L'=') {
            ++equals;
        }
        if (*equals != L'=') {
            continue;
        }
        if (!first) {
            CatString(dst, dst_chars, L", ");
        }
        *equals = 0;
        AppendPythonString(dst, dst_chars, entry);
        CatString(dst, dst_chars, L": ");
        AppendPythonString(dst, dst_chars, equals + 1);
        *equals = L'=';
        first = FALSE;
    }
    if (free_env) {
        free_env(block);
    }
    AppendLine(dst, dst_chars, L"}");
}

static void RefreshDump() {
    WCHAR text[MAX_DUMP_TEXT];
    CopyString(text, MAX_DUMP_TEXT, L"");
    AppendCurrentDirectory(text, MAX_DUMP_TEXT);
    AppendCommandLineArgs(text, MAX_DUMP_TEXT);
    AppendEnvironment(text, MAX_DUMP_TEXT);
    SetWindowTextW(g_dump, text);
}

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
    int button_h = height / 7;
    int dump_top = height / 2;
    int left = (width - button_w) / 2;
    int top = height / 12;
    MoveWindow(g_inavi, left, top, button_w, button_h, TRUE);
    MoveWindow(g_explorer, left, top + button_h + height / 10, button_w, button_h, TRUE);
    MoveWindow(g_dump, 0, dump_top, width, height - dump_top, TRUE);
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
        g_dump = CreateWindowW(L"EDIT", L"",
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
            0, 0, 0, 0, hwnd, (HMENU)EDIT_DUMP, g_instance, NULL);
        RefreshDump();
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
