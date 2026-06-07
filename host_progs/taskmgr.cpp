#include <windows.h>
#include <tlhelp32.h>
#include "dyn_resolve_helper.h"

#pragma comment(lib, "coredll.lib")

#define TIMER_REFRESH 1
#define EDIT_STATUS 1001
#define MAX_TEXT 8192
#define MAX_PATH_CE 260

static HINSTANCE g_instance;
static HWND g_main;
static HWND g_text;
static DWORD g_last_tick;
static DWORD g_last_idle;
static DWORD g_static_idle_samples;

static void CopyString(WCHAR *dst, int dst_chars, const WCHAR *src) {
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

static void CatString(WCHAR *dst, int dst_chars, const WCHAR *src) {
    int len = lstrlenW(dst);
    CopyString(dst + len, dst_chars - len, src);
}

static void AppendLine(WCHAR *dst, int dst_chars, const WCHAR *line) {
    CatString(dst, dst_chars, line);
    CatString(dst, dst_chars, L"\r\n");
}

static void AppendFormat2(WCHAR *dst, int dst_chars, LPCWSTR fmt, DWORD a, DWORD b) {
    WCHAR line[160];
    wsprintfW(line, fmt, a, b);
    AppendLine(dst, dst_chars, line);
}

static BOOL QueryRegistryDword(LPCWSTR subkey, LPCWSTR value_name, DWORD *out) {
    HKEY key;
    DWORD size = sizeof(*out);
    DWORD type = 0;
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, 0, 0, &key) != ERROR_SUCCESS) {
        return FALSE;
    }
    BOOL ok = RegQueryValueExW(key, value_name, NULL, &type, (LPBYTE)out, &size) == ERROR_SUCCESS
        && type == REG_DWORD;
    RegCloseKey(key);
    return ok;
}

static BOOL QueryRegistryString(LPCWSTR subkey, LPCWSTR value_name, WCHAR *out, DWORD out_chars) {
    HKEY key;
    DWORD size = out_chars * sizeof(WCHAR);
    DWORD type = 0;
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, 0, 0, &key) != ERROR_SUCCESS) {
        return FALSE;
    }
    BOOL ok = RegQueryValueExW(key, value_name, NULL, &type, (LPBYTE)out, &size) == ERROR_SUCCESS
        && (type == REG_SZ || type == REG_EXPAND_SZ) && out[0] != 0;
    RegCloseKey(key);
    return ok;
}

static DWORD QueryCpuMHz() {
    DWORD value = 0;
    if (!QueryRegistryDword(L"HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0", L"~MHz", &value)) {
        QueryRegistryDword(L"System\\CentralProcessor\\0", L"~MHz", &value);
    }
    return value;
}

static void QueryCpuName(WCHAR *out, int out_chars) {
    if (QueryRegistryString(L"HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
            L"ProcessorNameString", out, out_chars)) {
        return;
    }
    if (QueryRegistryString(L"System\\CentralProcessor\\0", L"ProcessorNameString", out, out_chars)) {
        return;
    }
    if (QueryRegistryString(L"System\\CentralProcessor\\0", L"Identifier", out, out_chars)) {
        return;
    }
    CopyString(out, out_chars, L"MIPS R4000");
}

static DWORD QueryIdleMillis(BOOL *available) {
    FARPROC proc = ResolveRuntimeProc(L"coredll.dll", L"GetIdleTime");
    if (!proc) {
        *available = FALSE;
        return 0;
    }
    typedef DWORD (WINAPI *PFN_GetIdleTime)();
    *available = TRUE;
    return ((PFN_GetIdleTime)proc)();
}

static DWORD EstimateCpuUsage(BOOL *available) {
    DWORD now_tick = GetTickCount();
    DWORD now_idle = QueryIdleMillis(available);
    DWORD usage = 0;
    if (!*available) {
        return 0;
    }
    if (g_last_tick && now_tick != g_last_tick && now_idle >= g_last_idle) {
        DWORD elapsed = now_tick - g_last_tick;
        DWORD idle = now_idle - g_last_idle;
        if (elapsed >= 500 && idle == 0 && now_idle == g_last_idle) {
            ++g_static_idle_samples;
            if (g_static_idle_samples >= 2) {
                *available = FALSE;
                g_last_tick = now_tick;
                g_last_idle = now_idle;
                return 0;
            }
        } else {
            g_static_idle_samples = 0;
        }
        usage = idle >= elapsed ? 0 : ((elapsed - idle) * 100) / elapsed;
        if (usage > 100) {
            usage = 100;
        }
    }
    g_last_tick = now_tick;
    g_last_idle = now_idle;
    return usage;
}

static void AppendCpu(WCHAR *text, int text_chars) {
    SYSTEM_INFO si;
    WCHAR cpu_name[96];
    DWORD mhz = QueryCpuMHz();
    BOOL cpu_usage_available = FALSE;
    DWORD usage = EstimateCpuUsage(&cpu_usage_available);
    ZeroMemory(&si, sizeof(si));
    GetSystemInfo(&si);
    QueryCpuName(cpu_name, 96);
    AppendLine(text, text_chars, L"CPU");
    if (mhz) {
        WCHAR line[160];
        wsprintfW(line, L"CPU: %s (%lu core) @ %lu.%03lu GHz",
            cpu_name,
            si.dwNumberOfProcessors ? si.dwNumberOfProcessors : 1,
            mhz / 1000, mhz % 1000);
        AppendLine(text, text_chars, line);
    } else {
        WCHAR line[120];
        wsprintfW(line, L"CPU: %s (%lu core) @ N/A",
            cpu_name,
            si.dwNumberOfProcessors ? si.dwNumberOfProcessors : 1);
        AppendLine(text, text_chars, line);
    }
    if (cpu_usage_available) {
        AppendFormat2(text, text_chars, L"CPU Usage (%%): %lu", usage, 0);
    } else {
        AppendLine(text, text_chars, L"CPU Usage (%): N/A");
    }
    AppendLine(text, text_chars, L"");
}

static void AppendMemory(WCHAR *text, int text_chars) {
    MEMORYSTATUS ms;
    DWORD total_mb_x10;
    DWORD avail_mb_x10;
    ZeroMemory(&ms, sizeof(ms));
    ms.dwLength = sizeof(ms);
    GlobalMemoryStatus(&ms);
    total_mb_x10 = (DWORD)(((ULONGLONG)ms.dwTotalPhys * 10) / (1024 * 1024));
    avail_mb_x10 = (DWORD)(((ULONGLONG)ms.dwAvailPhys * 10) / (1024 * 1024));
    WCHAR line[160];
    wsprintfW(line, L"Memory Total/Avail: %lu.%luMB / %lu.%luMB, %lu%%",
        total_mb_x10 / 10, total_mb_x10 % 10,
        avail_mb_x10 / 10, avail_mb_x10 % 10,
        ms.dwMemoryLoad);
    AppendLine(text, text_chars, line);
    AppendLine(text, text_chars, L"");
}

static void AppendOneDisk(WCHAR *text, int text_chars, int index, LPCWSTR path) {
    ULARGE_INTEGER free_to_caller;
    ULARGE_INTEGER total;
    ULARGE_INTEGER free_total;
    WCHAR line[220];
    if (GetDiskFreeSpaceExW(path, &free_to_caller, &total, &free_total)) {
        DWORD avail_mb = (DWORD)(free_to_caller.QuadPart / (1024 * 1024));
        DWORD total_mb = (DWORD)(total.QuadPart / (1024 * 1024));
        wsprintfW(line, L"Part #%d (%s) %lu/%lu MB", index, path, avail_mb, total_mb);
    } else {
        wsprintfW(line, L"Part #%d (%s) unavailable, error=%lu", index, path, GetLastError());
    }
    AppendLine(text, text_chars, line);
}

static void AppendDisks(WCHAR *text, int text_chars) {
    WIN32_FIND_DATAW fd;
    HANDLE find;
    int index = 1;
    WCHAR path[MAX_PATH_CE];
    AppendLine(text, text_chars, L"Storage");
    AppendOneDisk(text, text_chars, index++, L"\\");
    find = FindFirstFileW(L"\\*", &fd);
    if (find != INVALID_HANDLE_VALUE) {
        do {
            if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)) {
                continue;
            }
            if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_TEMPORARY)) {
                continue;
            }
            if (fd.cFileName[0] == L'.') {
                continue;
            }
            CopyString(path, MAX_PATH_CE, L"\\");
            CatString(path, MAX_PATH_CE, fd.cFileName);
            AppendOneDisk(text, text_chars, index++, path);
        } while (FindNextFileW(find, &fd));
        FindClose(find);
    }
    AppendLine(text, text_chars, L"");
}

static void AppendProcesses(WCHAR *text, int text_chars) {
    HANDLE snap;
    PROCESSENTRY32 pe;
    AppendLine(text, text_chars, L"Processes");
    AppendLine(text, text_chars, L"Name                         PID(hex)    Parent     Thrd  Memory  CPU");
    snap = DynCreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snap == INVALID_HANDLE_VALUE) {
        WCHAR line[120];
        wsprintfW(line, L"Process snapshot unavailable, error=%lu", GetLastError());
        AppendLine(text, text_chars, line);
        return;
    }
    ZeroMemory(&pe, sizeof(pe));
    pe.dwSize = sizeof(pe);
    if (DynProcess32First(snap, &pe)) {
        do {
            WCHAR line[260];
            wsprintfW(line, L"%-28s 0x%08lx 0x%08lx %4lu  N/A     N/A",
                pe.szExeFile,
                pe.th32ProcessID,
                pe.th32ParentProcessID,
                pe.cntThreads);
            AppendLine(text, text_chars, line);
            ZeroMemory(&pe, sizeof(pe));
            pe.dwSize = sizeof(pe);
        } while (DynProcess32Next(snap, &pe));
    } else {
        WCHAR line[120];
        wsprintfW(line, L"Process32First unavailable, error=%lu", GetLastError());
        AppendLine(text, text_chars, line);
    }
    CloseHandle(snap);
}

static void RefreshText() {
    WCHAR text[MAX_TEXT];
    CopyString(text, MAX_TEXT, L"Windows CE Task Manager\r\n\r\n");
    AppendCpu(text, MAX_TEXT);
    AppendMemory(text, MAX_TEXT);
    AppendDisks(text, MAX_TEXT);
    AppendProcesses(text, MAX_TEXT);
    SetWindowTextW(g_text, text);
}

static void Layout(HWND hwnd) {
    RECT rc;
    GetClientRect(hwnd, &rc);
    MoveWindow(g_text, 0, 0, rc.right, rc.bottom, TRUE);
}

static LRESULT CALLBACK MainProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    switch (msg) {
    case WM_CREATE:
        g_text = CreateWindowW(L"EDIT", L"",
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
            0, 0, 0, 0, hwnd, (HMENU)EDIT_STATUS, g_instance, NULL);
        RefreshText();
        SetTimer(hwnd, TIMER_REFRESH, 1000, NULL);
        Layout(hwnd);
        return 0;
    case WM_TIMER:
        if (wp == TIMER_REFRESH) {
            RefreshText();
            return 0;
        }
        break;
    case WM_SIZE:
        Layout(hwnd);
        return 0;
    case WM_DESTROY:
        KillTimer(hwnd, TIMER_REFRESH);
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
    wc.lpszClassName = L"FakeCeTaskMgrWindow";
    return RegisterClassW(&wc) != 0;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int show_cmd) {
    MSG msg;
    g_instance = hInstance;
    if (!RegisterMainClass()) {
        return 1;
    }
    g_main = CreateWindowW(L"FakeCeTaskMgrWindow", L"Task Manager",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT, 640, 420,
        NULL, NULL, hInstance, NULL);
    if (!g_main) {
        return 1;
    }
    ShowWindow(g_main, show_cmd);
    UpdateWindow(g_main);
    while (GetMessageW(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    return (int)msg.wParam;
}
