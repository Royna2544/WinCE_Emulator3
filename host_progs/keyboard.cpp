// keyboard.cpp — software QWERTY keyboard for Windows CE (SIP substitute).
//
// The window never takes activation (WS_EX_NOACTIVATE + MA_NOACTIVATE), so the
// previously focused application keeps the system focus while keys are tapped.
// Key taps are injected with keybd_event(), which delivers to the focused
// window of the foreground application — the only injection API available on
// CE (there is no SendInput). Characters map to virtual keys with VkKeyScanW
// so the active keyboard layout decides shift pairing on a real device.
//
// Constraints honored:
//  - QWERTY layout with a number row.
//  - Never full screen: docked strip, height capped well below half the
//    screen, narrow side margins.
//  - Positioned below the screen center (bottom-docked).

#include <windows.h>

#ifndef WS_EX_NOACTIVATE
#define WS_EX_NOACTIVATE 0x08000000L
#endif
#ifndef WM_MOUSEACTIVATE
#define WM_MOUSEACTIVATE 0x0021
#endif
#ifndef MA_NOACTIVATE
#define MA_NOACTIVATE 3
#endif
#ifndef VK_OEM_1
#define VK_OEM_1 0xBA
#define VK_OEM_PLUS 0xBB
#define VK_OEM_COMMA 0xBC
#define VK_OEM_MINUS 0xBD
#define VK_OEM_PERIOD 0xBE
#define VK_OEM_2 0xBF
#define VK_OEM_7 0xDE
#endif

#define KEY_ROWS 5
#define MAX_KEYS_PER_ROW 12

// Special key identifiers (stored in the `special` field).
#define KEYBOARD_SPECIAL_NONE 0
#define KEYBOARD_SPECIAL_BACKSPACE 1
#define KEYBOARD_SPECIAL_ENTER 2
#define KEYBOARD_SPECIAL_SHIFT 3
#define KEYBOARD_SPECIAL_TAB 4
#define KEYBOARD_SPECIAL_SPACE 5
#define KEYBOARD_SPECIAL_CLOSE 6

struct KeyboardKey {
    const WCHAR *label;        // base label (lowercase form)
    const WCHAR *shift_label;  // label when shift is latched (NULL = uppercase of label)
    WCHAR ch;                  // character to send (0 for specials)
    WCHAR shift_ch;            // character when shift is latched (0 = towupper-style)
    int special;               // KEYBOARD_SPECIAL_* id
    int weight;                // relative width (1 = one unit)
};

struct KeyboardRow {
    const KeyboardKey *keys;
    int count;
};

static const KeyboardKey g_row0[] = {
    {L"1", L"!", L'1', L'!', KEYBOARD_SPECIAL_NONE, 2},
    {L"2", L"@", L'2', L'@', KEYBOARD_SPECIAL_NONE, 2},
    {L"3", L"#", L'3', L'#', KEYBOARD_SPECIAL_NONE, 2},
    {L"4", L"$", L'4', L'$', KEYBOARD_SPECIAL_NONE, 2},
    {L"5", L"%", L'5', L'%', KEYBOARD_SPECIAL_NONE, 2},
    {L"6", L"^", L'6', L'^', KEYBOARD_SPECIAL_NONE, 2},
    {L"7", L"&", L'7', L'&', KEYBOARD_SPECIAL_NONE, 2},
    {L"8", L"*", L'8', L'*', KEYBOARD_SPECIAL_NONE, 2},
    {L"9", L"(", L'9', L'(', KEYBOARD_SPECIAL_NONE, 2},
    {L"0", L")", L'0', L')', KEYBOARD_SPECIAL_NONE, 2},
    {L"BS", NULL, 0, 0, KEYBOARD_SPECIAL_BACKSPACE, 3},
};

static const KeyboardKey g_row1[] = {
    {L"q", NULL, L'q', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"w", NULL, L'w', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"e", NULL, L'e', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"r", NULL, L'r', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"t", NULL, L't', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"y", NULL, L'y', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"u", NULL, L'u', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"i", NULL, L'i', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"o", NULL, L'o', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"p", NULL, L'p', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"-", L"_", L'-', L'_', KEYBOARD_SPECIAL_NONE, 2},
};

static const KeyboardKey g_row2[] = {
    {L"a", NULL, L'a', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"s", NULL, L's', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"d", NULL, L'd', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"f", NULL, L'f', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"g", NULL, L'g', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"h", NULL, L'h', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"j", NULL, L'j', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"k", NULL, L'k', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"l", NULL, L'l', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"Enter", NULL, 0, 0, KEYBOARD_SPECIAL_ENTER, 4},
};

static const KeyboardKey g_row3[] = {
    {L"Shift", NULL, 0, 0, KEYBOARD_SPECIAL_SHIFT, 3},
    {L"z", NULL, L'z', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"x", NULL, L'x', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"c", NULL, L'c', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"v", NULL, L'v', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"b", NULL, L'b', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"n", NULL, L'n', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L"m", NULL, L'm', 0, KEYBOARD_SPECIAL_NONE, 2},
    {L",", L"<", L',', L'<', KEYBOARD_SPECIAL_NONE, 2},
    {L".", L">", L'.', L'>', KEYBOARD_SPECIAL_NONE, 2},
    {L"/", L"?", L'/', L'?', KEYBOARD_SPECIAL_NONE, 2},
};

static const KeyboardKey g_row4[] = {
    {L"X", NULL, 0, 0, KEYBOARD_SPECIAL_CLOSE, 2},
    {L"Tab", NULL, 0, 0, KEYBOARD_SPECIAL_TAB, 3},
    {L"Space", NULL, 0, 0, KEYBOARD_SPECIAL_SPACE, 12},
    {L";", L":", L';', L':', KEYBOARD_SPECIAL_NONE, 2},
    {L"'", L"\"", L'\'', L'"', KEYBOARD_SPECIAL_NONE, 2},
    {L"=", L"+", L'=', L'+', KEYBOARD_SPECIAL_NONE, 2},
};

static const KeyboardRow g_rows[KEY_ROWS] = {
    {g_row0, sizeof(g_row0) / sizeof(g_row0[0])},
    {g_row1, sizeof(g_row1) / sizeof(g_row1[0])},
    {g_row2, sizeof(g_row2) / sizeof(g_row2[0])},
    {g_row3, sizeof(g_row3) / sizeof(g_row3[0])},
    {g_row4, sizeof(g_row4) / sizeof(g_row4[0])},
};

static HINSTANCE g_instance;
static HWND g_main;
static BOOL g_shift;
static int g_pressed_row = -1;
static int g_pressed_col = -1;
static RECT g_key_rects[KEY_ROWS][MAX_KEYS_PER_ROW];

static WCHAR ToUpperAscii(WCHAR ch) {
    if (ch >= L'a' && ch <= L'z') {
        return (WCHAR)(ch - L'a' + L'A');
    }
    return ch;
}

static void SendVirtualKey(BYTE vk, BOOL with_shift) {
    if (with_shift) {
        keybd_event(VK_SHIFT, 0, 0, 0);
    }
    keybd_event(vk, 0, 0, 0);
    keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
    if (with_shift) {
        keybd_event(VK_SHIFT, 0, KEYEVENTF_KEYUP, 0);
    }
}

// Static US-QWERTY character-to-virtual-key mapping. The on-screen keycaps
// are US QWERTY, so this table is exact for everything the user can tap;
// the CE 4.2 SDK does not declare VkKeyScanW.
static BOOL CharToVirtualKey(WCHAR ch, BYTE *vk, BOOL *with_shift) {
    static const struct {
        WCHAR ch;
        BYTE vk;
        BOOL shift;
    } table[] = {
        {L'!', '1', TRUE},          {L'@', '2', TRUE},
        {L'#', '3', TRUE},          {L'$', '4', TRUE},
        {L'%', '5', TRUE},          {L'^', '6', TRUE},
        {L'&', '7', TRUE},          {L'*', '8', TRUE},
        {L'(', '9', TRUE},          {L')', '0', TRUE},
        {L'-', VK_OEM_MINUS, FALSE}, {L'_', VK_OEM_MINUS, TRUE},
        {L'=', VK_OEM_PLUS, FALSE}, {L'+', VK_OEM_PLUS, TRUE},
        {L',', VK_OEM_COMMA, FALSE}, {L'<', VK_OEM_COMMA, TRUE},
        {L'.', VK_OEM_PERIOD, FALSE}, {L'>', VK_OEM_PERIOD, TRUE},
        {L'/', VK_OEM_2, FALSE},    {L'?', VK_OEM_2, TRUE},
        {L';', VK_OEM_1, FALSE},    {L':', VK_OEM_1, TRUE},
        {L'\'', VK_OEM_7, FALSE},   {L'"', VK_OEM_7, TRUE},
    };
    int i;
    if (ch >= L'0' && ch <= L'9') {
        *vk = (BYTE)ch;
        *with_shift = FALSE;
        return TRUE;
    }
    if (ch >= L'a' && ch <= L'z') {
        *vk = (BYTE)(ch - L'a' + 'A');
        *with_shift = FALSE;
        return TRUE;
    }
    if (ch >= L'A' && ch <= L'Z') {
        *vk = (BYTE)ch;
        *with_shift = TRUE;
        return TRUE;
    }
    for (i = 0; i < (int)(sizeof(table) / sizeof(table[0])); ++i) {
        if (table[i].ch == ch) {
            *vk = table[i].vk;
            *with_shift = table[i].shift;
            return TRUE;
        }
    }
    return FALSE;
}

static void SendCharacter(WCHAR ch) {
    BYTE vk = 0;
    BOOL with_shift = FALSE;
    if (CharToVirtualKey(ch, &vk, &with_shift)) {
        SendVirtualKey(vk, with_shift);
    }
}

static void SendKey(const KeyboardKey *key) {
    switch (key->special) {
        case KEYBOARD_SPECIAL_BACKSPACE:
            SendVirtualKey(VK_BACK, FALSE);
            return;
        case KEYBOARD_SPECIAL_ENTER:
            SendVirtualKey(VK_RETURN, FALSE);
            return;
        case KEYBOARD_SPECIAL_TAB:
            SendVirtualKey(VK_TAB, FALSE);
            return;
        case KEYBOARD_SPECIAL_SPACE:
            SendVirtualKey(VK_SPACE, FALSE);
            return;
        case KEYBOARD_SPECIAL_SHIFT:
            g_shift = !g_shift;
            InvalidateRect(g_main, NULL, FALSE);
            return;
        case KEYBOARD_SPECIAL_CLOSE:
            DestroyWindow(g_main);
            return;
        default:
            break;
    }
    WCHAR ch = key->ch;
    if (g_shift) {
        if (key->shift_ch) {
            ch = key->shift_ch;
        } else {
            ch = ToUpperAscii(ch);
        }
    }
    SendCharacter(ch);
    if (g_shift && key->special == KEYBOARD_SPECIAL_NONE) {
        // One-shot shift, like the CE SIP: release after one character.
        g_shift = FALSE;
        InvalidateRect(g_main, NULL, FALSE);
    }
}

static void LayoutKeys(int client_width, int client_height) {
    int row;
    int row_height = client_height / KEY_ROWS;
    for (row = 0; row < KEY_ROWS; ++row) {
        const KeyboardRow *r = &g_rows[row];
        int total_weight = 0;
        int col;
        int x = 0;
        int top = row * row_height;
        for (col = 0; col < r->count; ++col) {
            total_weight += r->keys[col].weight;
        }
        for (col = 0; col < r->count; ++col) {
            int w = (client_width * r->keys[col].weight) / total_weight;
            RECT *rect = &g_key_rects[row][col];
            rect->left = x;
            rect->top = top;
            rect->right = (col == r->count - 1) ? client_width : x + w;
            rect->bottom = top + row_height;
            x += w;
        }
    }
}

static const KeyboardKey *HitTest(int x, int y, int *out_row, int *out_col) {
    int row;
    POINT pt;
    pt.x = x;
    pt.y = y;
    for (row = 0; row < KEY_ROWS; ++row) {
        const KeyboardRow *r = &g_rows[row];
        int col;
        for (col = 0; col < r->count; ++col) {
            if (PtInRect(&g_key_rects[row][col], pt)) {
                *out_row = row;
                *out_col = col;
                return &r->keys[col];
            }
        }
    }
    return NULL;
}

static void PaintKeyboard(HDC dc, const RECT *client) {
    HBRUSH background = CreateSolidBrush(RGB(48, 48, 48));
    HBRUSH key_face = CreateSolidBrush(RGB(96, 96, 96));
    HBRUSH key_down = CreateSolidBrush(RGB(0, 120, 215));
    HBRUSH key_latched = CreateSolidBrush(RGB(160, 120, 0));
    HPEN edge = CreatePen(PS_SOLID, 1, RGB(24, 24, 24));
    HGDIOBJ old_pen = SelectObject(dc, edge);
    int row;

    FillRect(dc, client, background);
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, RGB(255, 255, 255));

    for (row = 0; row < KEY_ROWS; ++row) {
        const KeyboardRow *r = &g_rows[row];
        int col;
        for (col = 0; col < r->count; ++col) {
            const KeyboardKey *key = &r->keys[col];
            RECT rect = g_key_rects[row][col];
            HBRUSH face = key_face;
            const WCHAR *label = key->label;
            WCHAR upper[2];
            InflateRect(&rect, -1, -1);
            if (row == g_pressed_row && col == g_pressed_col) {
                face = key_down;
            } else if (key->special == KEYBOARD_SPECIAL_SHIFT && g_shift) {
                face = key_latched;
            }
            FillRect(dc, &rect, face);
            if (g_shift) {
                if (key->shift_label) {
                    label = key->shift_label;
                } else if (key->special == KEYBOARD_SPECIAL_NONE && key->ch >= L'a' &&
                           key->ch <= L'z') {
                    upper[0] = ToUpperAscii(key->ch);
                    upper[1] = 0;
                    label = upper;
                }
            }
            DrawTextW(dc, label, -1, &rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        }
    }

    SelectObject(dc, old_pen);
    DeleteObject(edge);
    DeleteObject(background);
    DeleteObject(key_face);
    DeleteObject(key_down);
    DeleteObject(key_latched);
}

static LRESULT CALLBACK KeyboardWndProc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
    switch (msg) {
        case WM_MOUSEACTIVATE:
            // Never steal activation/focus from the target application.
            return MA_NOACTIVATE;
        case WM_SIZE:
            LayoutKeys(LOWORD(lparam), HIWORD(lparam));
            return 0;
        case WM_LBUTTONDOWN: {
            int row = -1;
            int col = -1;
            const KeyboardKey *key =
                HitTest((int)(short)LOWORD(lparam), (int)(short)HIWORD(lparam), &row, &col);
            if (key) {
                g_pressed_row = row;
                g_pressed_col = col;
                SetCapture(hwnd);
                InvalidateRect(hwnd, &g_key_rects[row][col], FALSE);
                SendKey(key);
            }
            return 0;
        }
        case WM_LBUTTONUP:
            if (g_pressed_row >= 0) {
                RECT rect = g_key_rects[g_pressed_row][g_pressed_col];
                g_pressed_row = -1;
                g_pressed_col = -1;
                ReleaseCapture();
                InvalidateRect(hwnd, &rect, FALSE);
            }
            return 0;
        case WM_PAINT: {
            PAINTSTRUCT ps;
            RECT client;
            HDC dc = BeginPaint(hwnd, &ps);
            GetClientRect(hwnd, &client);
            // Recompute every paint: WM_SIZE delivery at creation varies
            // between CE builds, and the layout pass is trivially cheap.
            LayoutKeys(client.right - client.left, client.bottom - client.top);
            PaintKeyboard(dc, &client);
            EndPaint(hwnd, &ps);
            return 0;
        }
        case WM_ERASEBKGND:
            return 1;
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
        default:
            break;
    }
    return DefWindowProcW(hwnd, msg, wparam, lparam);
}

int WINAPI WinMain(HINSTANCE instance, HINSTANCE prev, LPWSTR cmd_line, int show) {
    WNDCLASSW wc;
    int screen_width;
    int screen_height;
    int width;
    int height;
    int x;
    int y;
    MSG msg;
    HANDLE mutex;

    (void)prev;
    (void)cmd_line;
    (void)show;

    mutex = CreateMutexW(NULL, FALSE, L"CEKeyboardSingleton");
    if (mutex && GetLastError() == ERROR_ALREADY_EXISTS) {
        // Already running: nothing to do.
        return 0;
    }

    g_instance = instance;

    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = KeyboardWndProc;
    wc.cbClsExtra = 0;
    wc.cbWndExtra = 0;
    wc.hInstance = instance;
    wc.hIcon = NULL;
    wc.hCursor = NULL;
    wc.hbrBackground = NULL;
    wc.lpszMenuName = NULL;
    wc.lpszClassName = L"CEKeyboard";
    if (!RegisterClassW(&wc)) {
        return 1;
    }

    // Bottom-docked strip: never full screen, top edge always below center.
    screen_width = GetSystemMetrics(SM_CXSCREEN);
    screen_height = GetSystemMetrics(SM_CYSCREEN);
    width = screen_width - 8;
    height = (screen_height * 2) / 5;
    if (height > (screen_height / 2) - 8) {
        height = (screen_height / 2) - 8;
    }
    x = (screen_width - width) / 2;
    y = screen_height - height - 2;

    g_main = CreateWindowExW(WS_EX_TOPMOST | WS_EX_NOACTIVATE, L"CEKeyboard", L"Keyboard",
                             WS_POPUP | WS_BORDER | WS_VISIBLE, x, y, width, height, NULL, NULL,
                             instance, NULL);
    if (!g_main) {
        return 1;
    }

    while (GetMessageW(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    if (mutex) {
        CloseHandle(mutex);
    }
    return (int)msg.wParam;
}
