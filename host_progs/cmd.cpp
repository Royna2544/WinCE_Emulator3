#include <windows.h>

#define CMD_EDIT_OUT 1001
#define CMD_EDIT_IN 1002
#define CMD_BUTTON_RUN 1003

#define MAX_CMD 512
#define MAX_PATH_CE 260
#define OUT_CHUNK 2048

static HINSTANCE g_instance;
static HWND g_main;
static HWND g_output;
static HWND g_input;
static HWND g_button;
static WNDPROC g_input_proc;
static WCHAR g_cwd[MAX_PATH_CE] = L"\\";

static void AppendText(const WCHAR *text);
static DWORD ExecuteCommand(const WCHAR *command, BOOL interactive);

static void CopyString(WCHAR *dst, int dst_chars, const WCHAR *src) {
    if (!dst || dst_chars <= 0) {
        return;
    }
    int i = 0;
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

static BOOL IsSpace(WCHAR ch) {
    return ch == L' ' || ch == L'\t' || ch == L'\r' || ch == L'\n';
}

static const WCHAR *SkipSpaces(const WCHAR *text) {
    while (text && IsSpace(*text)) {
        ++text;
    }
    return text;
}

static void TrimInPlace(WCHAR *text) {
    if (!text) {
        return;
    }
    WCHAR *start = text;
    while (IsSpace(*start)) {
        ++start;
    }
    if (start != text) {
        WCHAR *out = text;
        while (*start) {
            *out++ = *start++;
        }
        *out = 0;
    }
    int len = lstrlenW(text);
    while (len > 0 && IsSpace(text[len - 1])) {
        text[--len] = 0;
    }
}

static int CompareNoCase(const WCHAR *a, const WCHAR *b) {
    while (*a && *b) {
        WCHAR ca = *a;
        WCHAR cb = *b;
        if (ca >= L'A' && ca <= L'Z') {
            ca = (WCHAR)(ca + (L'a' - L'A'));
        }
        if (cb >= L'A' && cb <= L'Z') {
            cb = (WCHAR)(cb + (L'a' - L'A'));
        }
        if (ca != cb) {
            return ca - cb;
        }
        ++a;
        ++b;
    }
    return *a - *b;
}

static BOOL StartsWithSwitch(const WCHAR *text, WCHAR sw) {
    if (!text || (text[0] != L'/' && text[0] != L'-')) {
        return FALSE;
    }
    WCHAR ch = text[1];
    if (ch >= L'a' && ch <= L'z') {
        ch = (WCHAR)(ch - (L'a' - L'A'));
    }
    if (sw >= L'a' && sw <= L'z') {
        sw = (WCHAR)(sw - (L'a' - L'A'));
    }
    return ch == sw && (text[2] == 0 || IsSpace(text[2]));
}

static const WCHAR *BaseName(const WCHAR *path) {
    const WCHAR *base = path;
    for (const WCHAR *p = path; p && *p; ++p) {
        if (*p == L'\\' || *p == L'/') {
            base = p + 1;
        }
    }
    return base;
}

static void NormalizeSlashes(WCHAR *text) {
    for (int i = 0; text && text[i]; ++i) {
        if (text[i] == L'/') {
            text[i] = L'\\';
        }
    }
}

static void ResolvePath(const WCHAR *arg, WCHAR *out, int out_chars) {
    WCHAR temp[MAX_PATH_CE];
    const WCHAR *input = SkipSpaces(arg);
    CopyString(temp, MAX_PATH_CE, input ? input : L"");
    TrimInPlace(temp);
    NormalizeSlashes(temp);

    if (temp[0] == 0) {
        CopyString(out, out_chars, g_cwd);
        return;
    }
    if (temp[0] == L'\\') {
        CopyString(out, out_chars, temp);
        return;
    }
    CopyString(out, out_chars, g_cwd);
    int len = lstrlenW(out);
    if (len == 0) {
        CopyString(out, out_chars, L"\\");
        len = 1;
    }
    if (out[len - 1] != L'\\') {
        CatString(out, out_chars, L"\\");
    }
    CatString(out, out_chars, temp);
}

static void ParentPath(WCHAR *path) {
    int len = lstrlenW(path);
    while (len > 1 && path[len - 1] == L'\\') {
        path[--len] = 0;
    }
    while (len > 1 && path[len - 1] != L'\\') {
        path[--len] = 0;
    }
    if (len > 1 && path[len - 1] == L'\\') {
        path[len - 1] = 0;
    }
    if (path[0] == 0) {
        CopyString(path, MAX_PATH_CE, L"\\");
    }
}

static BOOL ChangeDirectory(const WCHAR *arg) {
    WCHAR target[MAX_PATH_CE];
    WCHAR probe[MAX_PATH_CE];
    WIN32_FIND_DATAW fd;
    HANDLE find;

    if (!arg || !*SkipSpaces(arg)) {
        AppendText(g_cwd);
        AppendText(L"\r\n");
        return TRUE;
    }
    if (CompareNoCase(SkipSpaces(arg), L"..") == 0) {
        ParentPath(g_cwd);
        return TRUE;
    }

    ResolvePath(arg, target, MAX_PATH_CE);
    CopyString(probe, MAX_PATH_CE, target);
    find = FindFirstFileW(probe, &fd);
    if (find == INVALID_HANDLE_VALUE) {
        AppendText(L"The system cannot find the path specified.\r\n");
        return FALSE;
    }
    FindClose(find);
    if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)) {
        AppendText(L"The directory name is invalid.\r\n");
        return FALSE;
    }
    CopyString(g_cwd, MAX_PATH_CE, target);
    return TRUE;
}

static void AppendText(const WCHAR *text) {
    if (!g_output || !text) {
        return;
    }
    int end = GetWindowTextLengthW(g_output);
    SendMessageW(g_output, EM_SETSEL, end, end);
    SendMessageW(g_output, EM_REPLACESEL, FALSE, (LPARAM)text);
    SendMessageW(g_output, EM_SCROLLCARET, 0, 0);
}

static void AppendLineFormat(const WCHAR *a, const WCHAR *b, const WCHAR *c) {
    WCHAR line[OUT_CHUNK];
    CopyString(line, OUT_CHUNK, a ? a : L"");
    if (b) {
        CatString(line, OUT_CHUNK, b);
    }
    if (c) {
        CatString(line, OUT_CHUNK, c);
    }
    CatString(line, OUT_CHUNK, L"\r\n");
    AppendText(line);
}

static DWORD CmdDir(const WCHAR *arg) {
    WCHAR path[MAX_PATH_CE];
    WCHAR pattern[MAX_PATH_CE];
    WIN32_FIND_DATAW fd;
    HANDLE find;
    DWORD count = 0;

    ResolvePath(arg && *SkipSpaces(arg) ? arg : L".", path, MAX_PATH_CE);
    CopyString(pattern, MAX_PATH_CE, path);
    int len = lstrlenW(pattern);
    if (len > 0 && pattern[len - 1] != L'\\') {
        CatString(pattern, MAX_PATH_CE, L"\\");
    }
    CatString(pattern, MAX_PATH_CE, L"*");

    AppendLineFormat(L" Directory of ", path, NULL);
    find = FindFirstFileW(pattern, &fd);
    if (find == INVALID_HANDLE_VALUE) {
        AppendText(L"File Not Found\r\n");
        return 1;
    }
    do {
        if (CompareNoCase(fd.cFileName, L".") == 0 || CompareNoCase(fd.cFileName, L"..") == 0) {
            continue;
        }
        if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            AppendLineFormat(L"<DIR>          ", fd.cFileName, NULL);
        } else {
            WCHAR size[48];
            wsprintfW(size, L"%10lu  ", fd.nFileSizeLow);
            AppendLineFormat(size, fd.cFileName, NULL);
        }
        ++count;
    } while (FindNextFileW(find, &fd));
    FindClose(find);
    if (count == 0) {
        AppendText(L"File Not Found\r\n");
        return 1;
    }
    return 0;
}

static DWORD CmdType(const WCHAR *arg) {
    WCHAR path[MAX_PATH_CE];
    BYTE buffer[512];
    WCHAR wide[513];
    DWORD read = 0;
    HANDLE file;

    ResolvePath(arg, path, MAX_PATH_CE);
    file = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    if (file == INVALID_HANDLE_VALUE) {
        AppendText(L"The system cannot find the file specified.\r\n");
        return 1;
    }
    while (ReadFile(file, buffer, sizeof(buffer), &read, NULL) && read > 0) {
        for (DWORD i = 0; i < read; ++i) {
            wide[i] = (buffer[i] == '\n') ? L'\n' : (WCHAR)buffer[i];
        }
        wide[read] = 0;
        AppendText(wide);
    }
    CloseHandle(file);
    AppendText(L"\r\n");
    return 0;
}

static DWORD CmdCopy(const WCHAR *args) {
    WCHAR src[MAX_PATH_CE];
    WCHAR dst[MAX_PATH_CE];
    WCHAR first[MAX_PATH_CE];
    const WCHAR *rest = SkipSpaces(args);
    int i = 0;
    while (*rest && !IsSpace(*rest) && i < MAX_PATH_CE - 1) {
        first[i++] = *rest++;
    }
    first[i] = 0;
    rest = SkipSpaces(rest);
    if (!first[0] || !*rest) {
        AppendText(L"Syntax: copy source destination\r\n");
        return 1;
    }
    ResolvePath(first, src, MAX_PATH_CE);
    ResolvePath(rest, dst, MAX_PATH_CE);
    if (!CopyFileW(src, dst, FALSE)) {
        AppendText(L"        0 file(s) copied.\r\n");
        return 1;
    }
    AppendText(L"        1 file(s) copied.\r\n");
    return 0;
}

static DWORD CmdDelete(const WCHAR *arg) {
    WCHAR path[MAX_PATH_CE];
    ResolvePath(arg, path, MAX_PATH_CE);
    if (!DeleteFileW(path)) {
        AppendText(L"Could Not Find ");
        AppendText(path);
        AppendText(L"\r\n");
        return 1;
    }
    return 0;
}

static DWORD CmdMkdir(const WCHAR *arg) {
    WCHAR path[MAX_PATH_CE];
    ResolvePath(arg, path, MAX_PATH_CE);
    if (!CreateDirectoryW(path, NULL)) {
        AppendText(L"A subdirectory or file already exists.\r\n");
        return 1;
    }
    return 0;
}

static DWORD CmdRmdir(const WCHAR *arg) {
    WCHAR path[MAX_PATH_CE];
    ResolvePath(arg, path, MAX_PATH_CE);
    if (!RemoveDirectoryW(path)) {
        AppendText(L"The directory is not empty or does not exist.\r\n");
        return 1;
    }
    return 0;
}

static DWORD RunExternal(const WCHAR *command) {
    WCHAR exe[MAX_PATH_CE];
    WCHAR cmdline[MAX_CMD];
    PROCESS_INFORMATION pi;
    DWORD exit_code = 0;
    ZeroMemory(&pi, sizeof(pi));

    CopyString(cmdline, MAX_CMD, command);
    const WCHAR *space = SkipSpaces(command);
    int i = 0;
    while (*space && !IsSpace(*space) && i < MAX_PATH_CE - 1) {
        exe[i++] = *space++;
    }
    exe[i] = 0;
    ResolvePath(exe, exe, MAX_PATH_CE);

    if (!CreateProcessW(exe, cmdline, NULL, NULL, FALSE, 0, NULL, NULL, NULL, &pi)) {
        AppendText(L"'");
        AppendText(command);
        AppendText(L"' is not recognized as an internal or external command.\r\n");
        return 1;
    }
    WaitForSingleObject(pi.hProcess, INFINITE);
    GetExitCodeProcess(pi.hProcess, &exit_code);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    return exit_code;
}

static DWORD ExecuteCommand(const WCHAR *command, BOOL interactive) {
    WCHAR temp[MAX_CMD];
    WCHAR name[64];
    const WCHAR *args;
    int i = 0;

    CopyString(temp, MAX_CMD, command);
    TrimInPlace(temp);
    if (!temp[0]) {
        return 0;
    }
    if (interactive) {
        AppendText(g_cwd);
        AppendText(L">");
        AppendText(temp);
        AppendText(L"\r\n");
    }

    args = temp;
    while (*args && !IsSpace(*args) && i < 63) {
        name[i++] = *args++;
    }
    name[i] = 0;
    args = SkipSpaces(args);

    if (CompareNoCase(name, L"cd") == 0 || CompareNoCase(name, L"chdir") == 0) {
        return ChangeDirectory(args) ? 0 : 1;
    }
    if (CompareNoCase(name, L"dir") == 0) {
        return CmdDir(args);
    }
    if (CompareNoCase(name, L"type") == 0) {
        return CmdType(args);
    }
    if (CompareNoCase(name, L"echo") == 0) {
        AppendText(args);
        AppendText(L"\r\n");
        return 0;
    }
    if (CompareNoCase(name, L"copy") == 0) {
        return CmdCopy(args);
    }
    if (CompareNoCase(name, L"del") == 0 || CompareNoCase(name, L"erase") == 0) {
        return CmdDelete(args);
    }
    if (CompareNoCase(name, L"md") == 0 || CompareNoCase(name, L"mkdir") == 0) {
        return CmdMkdir(args);
    }
    if (CompareNoCase(name, L"rd") == 0 || CompareNoCase(name, L"rmdir") == 0) {
        return CmdRmdir(args);
    }
    if (CompareNoCase(name, L"cls") == 0) {
        SetWindowTextW(g_output, L"");
        return 0;
    }
    if (CompareNoCase(name, L"pwd") == 0) {
        AppendText(g_cwd);
        AppendText(L"\r\n");
        return 0;
    }
    if (CompareNoCase(name, L"exit") == 0) {
        PostQuitMessage(0);
        return 0;
    }
    return RunExternal(temp);
}

static void RunInputCommand() {
    WCHAR command[MAX_CMD];
    GetWindowTextW(g_input, command, MAX_CMD);
    SetWindowTextW(g_input, L"");
    ExecuteCommand(command, TRUE);
}

static LRESULT CALLBACK InputProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_KEYDOWN && wp == VK_RETURN) {
        RunInputCommand();
        return 0;
    }
    return CallWindowProcW(g_input_proc, hwnd, msg, wp, lp);
}

static void Layout(HWND hwnd) {
    RECT rc;
    GetClientRect(hwnd, &rc);
    int button_w = 64;
    int input_h = 24;
    MoveWindow(g_output, 0, 0, rc.right, rc.bottom - input_h, TRUE);
    MoveWindow(g_input, 0, rc.bottom - input_h, rc.right - button_w, input_h, TRUE);
    MoveWindow(g_button, rc.right - button_w, rc.bottom - input_h, button_w, input_h, TRUE);
}

static LRESULT CALLBACK MainProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    switch (msg) {
    case WM_CREATE:
        g_output = CreateWindowW(L"EDIT", L"",
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_LEFT | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
            0, 0, 0, 0, hwnd, (HMENU)CMD_EDIT_OUT, g_instance, NULL);
        g_input = CreateWindowW(L"EDIT", L"",
            WS_CHILD | WS_VISIBLE | ES_LEFT | ES_AUTOHSCROLL,
            0, 0, 0, 0, hwnd, (HMENU)CMD_EDIT_IN, g_instance, NULL);
        g_button = CreateWindowW(L"BUTTON", L"Run",
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            0, 0, 0, 0, hwnd, (HMENU)CMD_BUTTON_RUN, g_instance, NULL);
        g_input_proc = (WNDPROC)SetWindowLongW(g_input, GWL_WNDPROC, (LONG)InputProc);
        AppendText(L"Microsoft Windows CE [cmd]\r\n");
        AppendText(L"Type 'exit' to close.\r\n\r\n");
        Layout(hwnd);
        SetFocus(g_input);
        return 0;
    case WM_SIZE:
        Layout(hwnd);
        return 0;
    case WM_COMMAND:
        if (LOWORD(wp) == CMD_BUTTON_RUN) {
            RunInputCommand();
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
    wc.lpszClassName = L"FakeCeCmdWindow";
    return RegisterClassW(&wc) != 0;
}

static DWORD RunScriptLine(const WCHAR *line) {
    WCHAR part[MAX_CMD];
    DWORD last = 0;
    int out = 0;
    for (int i = 0;; ++i) {
        WCHAR ch = line[i];
        if (ch == L'&' || ch == 0) {
            part[out] = 0;
            last = ExecuteCommand(part, FALSE);
            out = 0;
            if (ch == 0) {
                break;
            }
        } else if (out < MAX_CMD - 1) {
            part[out++] = ch;
        }
    }
    return last;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR command_line, int show_cmd) {
    MSG msg;
    DWORD exit_code = 0;
    BOOL keep_open = TRUE;
    const WCHAR *command = SkipSpaces(command_line);
    g_instance = hInstance;

    if (StartsWithSwitch(command, L'C')) {
        keep_open = FALSE;
        command = SkipSpaces(command + 2);
        return (int)RunScriptLine(command);
    }
    if (StartsWithSwitch(command, L'K')) {
        keep_open = TRUE;
        command = SkipSpaces(command + 2);
    }

    if (!RegisterMainClass()) {
        return 1;
    }
    g_main = CreateWindowW(L"FakeCeCmdWindow", L"Command Prompt",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        CW_USEDEFAULT, CW_USEDEFAULT, 640, 360,
        NULL, NULL, hInstance, NULL);
    if (!g_main) {
        return 1;
    }
    ShowWindow(g_main, show_cmd);
    UpdateWindow(g_main);

    if (command && *command) {
        exit_code = RunScriptLine(command);
        if (!keep_open) {
            return (int)exit_code;
        }
    }

    while (GetMessageW(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    return (int)msg.wParam;
}
