// rfb_server.cpp — minimal RFB 3.3 (VNC) server for Windows CE, port 5900.
//
// Scope: 1:1 screen mirror plus touch and keyboard injection.
//  - Protocol: RFB 003.003, security type None, Raw encoding only, single
//    client at a time.
//  - Capture: BitBlt from the display DC into an RGB565 DIB section, then
//    converted per-pixel to the client's requested true-colour format.
//  - Pointer: button-1 transitions map to mouse_event(MOUSEEVENTF_ABSOLUTE)
//    resolved at runtime via GetProcAddress because not every CE image
//    exports mouse_event; mirroring still works without it.
//  - Keys: keysym -> VK for ASCII and common control keys via keybd_event.
//
// All RFB wire integers are big-endian.

#include <windows.h>
#include <winsock2.h>

#define RFB_PORT 5900

#ifndef MOUSEEVENTF_MOVE
#define MOUSEEVENTF_MOVE 0x0001
#endif
#ifndef MOUSEEVENTF_LEFTDOWN
#define MOUSEEVENTF_LEFTDOWN 0x0002
#endif
#ifndef MOUSEEVENTF_LEFTUP
#define MOUSEEVENTF_LEFTUP 0x0004
#endif
#ifndef MOUSEEVENTF_ABSOLUTE
#define MOUSEEVENTF_ABSOLUTE 0x8000
#endif

typedef void(WINAPI *MouseEventProc)(DWORD flags, DWORD dx, DWORD dy, DWORD data, DWORD extra);

struct PixelFormat {
    BYTE bits_per_pixel;
    BYTE depth;
    BYTE big_endian;
    BYTE true_colour;
    WORD red_max;
    WORD green_max;
    WORD blue_max;
    BYTE red_shift;
    BYTE green_shift;
    BYTE blue_shift;
};

static int g_screen_w;
static int g_screen_h;
static HDC g_capture_dc;
static HBITMAP g_capture_bitmap;
static WORD *g_capture_pixels; // RGB565 rows, top-down
static PixelFormat g_client_format;
static BYTE g_last_buttons;
static MouseEventProc g_mouse_event;

static WORD SwapU16(WORD v) {
    return (WORD)((v >> 8) | (v << 8));
}

static DWORD SwapU32(DWORD v) {
    return ((v >> 24) & 0xff) | ((v >> 8) & 0xff00) | ((v << 8) & 0xff0000) | (v << 24);
}

static int RecvAll(SOCKET s, void *buffer, int len) {
    char *p = (char *)buffer;
    int total = 0;
    while (total < len) {
        int got = recv(s, p + total, len - total, 0);
        if (got <= 0) {
            return 0;
        }
        total += got;
    }
    return 1;
}

static int SendAll(SOCKET s, const void *data, int len) {
    const char *p = (const char *)data;
    int total = 0;
    while (total < len) {
        int sent = send(s, p + total, len - total, 0);
        if (sent <= 0) {
            return 0;
        }
        total += sent;
    }
    return 1;
}

static BOOL CreateCaptureSurface(void) {
    struct {
        BITMAPINFOHEADER header;
        DWORD masks[3];
    } info;
    HDC screen = GetDC(NULL);

    g_screen_w = GetSystemMetrics(SM_CXSCREEN);
    g_screen_h = GetSystemMetrics(SM_CYSCREEN);

    memset(&info, 0, sizeof(info));
    info.header.biSize = sizeof(BITMAPINFOHEADER);
    info.header.biWidth = g_screen_w;
    info.header.biHeight = -g_screen_h; // top-down
    info.header.biPlanes = 1;
    info.header.biBitCount = 16;
    info.header.biCompression = BI_BITFIELDS;
    info.masks[0] = 0xF800;
    info.masks[1] = 0x07E0;
    info.masks[2] = 0x001F;

    g_capture_dc = CreateCompatibleDC(screen);
    g_capture_bitmap = CreateDIBSection(screen, (BITMAPINFO *)&info, DIB_RGB_COLORS,
                                        (void **)&g_capture_pixels, NULL, 0);
    ReleaseDC(NULL, screen);
    if (!g_capture_dc || !g_capture_bitmap || !g_capture_pixels) {
        return FALSE;
    }
    SelectObject(g_capture_dc, g_capture_bitmap);
    return TRUE;
}

static void CaptureScreen(void) {
    HDC screen = GetDC(NULL);
    BitBlt(g_capture_dc, 0, 0, g_screen_w, g_screen_h, screen, 0, 0, SRCCOPY);
    ReleaseDC(NULL, screen);
}

// Expand a 565 component into the client's component range.
static DWORD ScaleComponent(DWORD value, DWORD src_max, DWORD dst_max) {
    if (src_max == 0) {
        return 0;
    }
    return (value * dst_max + src_max / 2) / src_max;
}

// Convert one RGB565 pixel into the client pixel format (little-endian
// composed, then byte-swapped on emit when the client asked for big-endian).
static DWORD ConvertPixel(WORD rgb565) {
    DWORD r = (rgb565 >> 11) & 0x1f;
    DWORD g = (rgb565 >> 5) & 0x3f;
    DWORD b = rgb565 & 0x1f;
    return (ScaleComponent(r, 31, g_client_format.red_max) << g_client_format.red_shift) |
           (ScaleComponent(g, 63, g_client_format.green_max) << g_client_format.green_shift) |
           (ScaleComponent(b, 31, g_client_format.blue_max) << g_client_format.blue_shift);
}

// Send one FramebufferUpdate with a single Raw rectangle.
static int SendFrameUpdate(SOCKET s, int x, int y, int w, int h) {
    BYTE header[16];
    int bytes_per_pixel = g_client_format.bits_per_pixel / 8;
    int row;
    char *row_buffer;

    if (x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > g_screen_w || y + h > g_screen_h) {
        x = 0;
        y = 0;
        w = g_screen_w;
        h = g_screen_h;
    }
    CaptureScreen();

    header[0] = 0; // FramebufferUpdate
    header[1] = 0;
    *(WORD *)(header + 2) = SwapU16(1); // one rectangle
    *(WORD *)(header + 4) = SwapU16((WORD)x);
    *(WORD *)(header + 6) = SwapU16((WORD)y);
    *(WORD *)(header + 8) = SwapU16((WORD)w);
    *(WORD *)(header + 10) = SwapU16((WORD)h);
    *(DWORD *)(header + 12) = SwapU32(0); // Raw encoding
    if (!SendAll(s, header, 16)) {
        return 0;
    }

    row_buffer = (char *)LocalAlloc(LMEM_FIXED, (UINT)(w * bytes_per_pixel));
    if (!row_buffer) {
        return 0;
    }
    for (row = 0; row < h; ++row) {
        const WORD *src = g_capture_pixels + (y + row) * g_screen_w + x;
        char *out = row_buffer;
        int col;
        for (col = 0; col < w; ++col) {
            DWORD pixel = ConvertPixel(src[col]);
            if (bytes_per_pixel == 4) {
                DWORD value = g_client_format.big_endian ? SwapU32(pixel) : pixel;
                *(DWORD *)out = value;
                out += 4;
            } else if (bytes_per_pixel == 2) {
                WORD value = g_client_format.big_endian ? SwapU16((WORD)pixel) : (WORD)pixel;
                *(WORD *)out = value;
                out += 2;
            } else {
                *out++ = (char)pixel;
            }
        }
        if (!SendAll(s, row_buffer, w * bytes_per_pixel)) {
            LocalFree(row_buffer);
            return 0;
        }
    }
    LocalFree(row_buffer);
    return 1;
}

static void InjectKeySym(DWORD keysym, BOOL down) {
    BYTE vk = 0;
    BOOL shift = FALSE;

    if (keysym >= 'a' && keysym <= 'z') {
        vk = (BYTE)(keysym - 'a' + 'A');
    } else if (keysym >= 'A' && keysym <= 'Z') {
        vk = (BYTE)keysym;
        shift = TRUE;
    } else if (keysym >= '0' && keysym <= '9') {
        vk = (BYTE)keysym;
    } else {
        switch (keysym) {
            case 0x0020: vk = VK_SPACE; break;
            case 0xFF08: vk = VK_BACK; break;
            case 0xFF09: vk = VK_TAB; break;
            case 0xFF0D: vk = VK_RETURN; break;
            case 0xFF1B: vk = VK_ESCAPE; break;
            case 0xFF50: vk = VK_HOME; break;
            case 0xFF51: vk = VK_LEFT; break;
            case 0xFF52: vk = VK_UP; break;
            case 0xFF53: vk = VK_RIGHT; break;
            case 0xFF54: vk = VK_DOWN; break;
            case 0xFF57: vk = VK_END; break;
            case 0xFFFF: vk = VK_DELETE; break;
            case 0xFFE1:
            case 0xFFE2: vk = VK_SHIFT; break;
            case 0xFFE3:
            case 0xFFE4: vk = VK_CONTROL; break;
            default: return; // unmapped keysym: ignore
        }
    }
    if (shift && down) {
        keybd_event(VK_SHIFT, 0, 0, 0);
    }
    keybd_event(vk, 0, down ? 0 : KEYEVENTF_KEYUP, 0);
    if (shift && !down) {
        keybd_event(VK_SHIFT, 0, KEYEVENTF_KEYUP, 0);
    }
}

static void InjectPointer(BYTE buttons, int x, int y) {
    DWORD nx;
    DWORD ny;
    if (!g_mouse_event || g_screen_w <= 1 || g_screen_h <= 1) {
        return;
    }
    nx = (DWORD)((x * 65535) / (g_screen_w - 1));
    ny = (DWORD)((y * 65535) / (g_screen_h - 1));
    g_mouse_event(MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE, nx, ny, 0, 0);
    if ((buttons & 1) && !(g_last_buttons & 1)) {
        g_mouse_event(MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_LEFTDOWN, nx, ny, 0, 0);
    } else if (!(buttons & 1) && (g_last_buttons & 1)) {
        g_mouse_event(MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_LEFTUP, nx, ny, 0, 0);
    }
    g_last_buttons = buttons;
}

static int Handshake(SOCKET s) {
    char version[13];
    DWORD security;
    BYTE shared;
    BYTE server_init[24 + 3];
    const char *name = "CE";

    if (!SendAll(s, "RFB 003.003\n", 12)) {
        return 0;
    }
    if (!RecvAll(s, version, 12)) {
        return 0;
    }
    security = SwapU32(1); // None
    if (!SendAll(s, &security, 4)) {
        return 0;
    }
    if (!RecvAll(s, &shared, 1)) {
        return 0;
    }

    // ServerInit: width, height, native RGB565 pixel format, name.
    memset(server_init, 0, sizeof(server_init));
    *(WORD *)(server_init + 0) = SwapU16((WORD)g_screen_w);
    *(WORD *)(server_init + 2) = SwapU16((WORD)g_screen_h);
    server_init[4] = 16; // bits per pixel
    server_init[5] = 16; // depth
    server_init[6] = 0;  // big endian
    server_init[7] = 1;  // true colour
    *(WORD *)(server_init + 8) = SwapU16(31);
    *(WORD *)(server_init + 10) = SwapU16(63);
    *(WORD *)(server_init + 12) = SwapU16(31);
    server_init[14] = 11; // red shift
    server_init[15] = 5;  // green shift
    server_init[16] = 0;  // blue shift
    *(DWORD *)(server_init + 20) = SwapU32(2); // name length
    server_init[24] = (BYTE)name[0];
    server_init[25] = (BYTE)name[1];
    if (!SendAll(s, server_init, 26)) {
        return 0;
    }

    // Until the client overrides it, send in the native format.
    g_client_format.bits_per_pixel = 16;
    g_client_format.depth = 16;
    g_client_format.big_endian = 0;
    g_client_format.true_colour = 1;
    g_client_format.red_max = 31;
    g_client_format.green_max = 63;
    g_client_format.blue_max = 31;
    g_client_format.red_shift = 11;
    g_client_format.green_shift = 5;
    g_client_format.blue_shift = 0;
    return 1;
}

static void ServeClient(SOCKET s) {
    g_last_buttons = 0;
    if (!Handshake(s)) {
        return;
    }
    for (;;) {
        BYTE type;
        if (!RecvAll(s, &type, 1)) {
            return;
        }
        switch (type) {
            case 0: { // SetPixelFormat
                BYTE payload[19];
                if (!RecvAll(s, payload, 19)) {
                    return;
                }
                g_client_format.bits_per_pixel = payload[3];
                g_client_format.depth = payload[4];
                g_client_format.big_endian = payload[5];
                g_client_format.true_colour = payload[6];
                g_client_format.red_max = SwapU16(*(WORD *)(payload + 7));
                g_client_format.green_max = SwapU16(*(WORD *)(payload + 9));
                g_client_format.blue_max = SwapU16(*(WORD *)(payload + 11));
                g_client_format.red_shift = payload[13];
                g_client_format.green_shift = payload[14];
                g_client_format.blue_shift = payload[15];
                if (g_client_format.bits_per_pixel != 8 &&
                    g_client_format.bits_per_pixel != 16 &&
                    g_client_format.bits_per_pixel != 32) {
                    return; // unsupportable format
                }
                break;
            }
            case 2: { // SetEncodings: read and ignore (Raw is always used)
                BYTE header[3];
                WORD count;
                WORD i;
                if (!RecvAll(s, header, 3)) {
                    return;
                }
                count = SwapU16(*(WORD *)(header + 1));
                for (i = 0; i < count; ++i) {
                    DWORD encoding;
                    if (!RecvAll(s, &encoding, 4)) {
                        return;
                    }
                }
                break;
            }
            case 3: { // FramebufferUpdateRequest
                BYTE payload[9];
                int x;
                int y;
                int w;
                int h;
                if (!RecvAll(s, payload, 9)) {
                    return;
                }
                x = SwapU16(*(WORD *)(payload + 1));
                y = SwapU16(*(WORD *)(payload + 3));
                w = SwapU16(*(WORD *)(payload + 5));
                h = SwapU16(*(WORD *)(payload + 7));
                if (!SendFrameUpdate(s, x, y, w, h)) {
                    return;
                }
                break;
            }
            case 4: { // KeyEvent
                BYTE payload[7];
                if (!RecvAll(s, payload, 7)) {
                    return;
                }
                InjectKeySym(SwapU32(*(DWORD *)(payload + 3)), payload[0] != 0);
                break;
            }
            case 5: { // PointerEvent
                BYTE payload[5];
                if (!RecvAll(s, payload, 5)) {
                    return;
                }
                InjectPointer(payload[0], SwapU16(*(WORD *)(payload + 1)),
                              SwapU16(*(WORD *)(payload + 3)));
                break;
            }
            case 6: { // ClientCutText: skip
                BYTE header[7];
                DWORD length;
                if (!RecvAll(s, header, 7)) {
                    return;
                }
                length = SwapU32(*(DWORD *)(header + 3));
                while (length > 0) {
                    char sink[256];
                    int want = length > sizeof(sink) ? (int)sizeof(sink) : (int)length;
                    if (!RecvAll(s, sink, want)) {
                        return;
                    }
                    length -= want;
                }
                break;
            }
            default:
                return; // unknown message: drop the client
        }
    }
}

int WINAPI WinMain(HINSTANCE instance, HINSTANCE prev, LPWSTR cmd_line, int show) {
    WSADATA wsa;
    SOCKET listener;
    struct sockaddr_in addr;
    HANDLE mutex;
    HMODULE coredll;

    (void)instance;
    (void)prev;
    (void)cmd_line;
    (void)show;

    mutex = CreateMutexW(NULL, FALSE, L"RfbServerSingleton");
    if (mutex && GetLastError() == ERROR_ALREADY_EXISTS) {
        return 0;
    }
    if (WSAStartup(MAKEWORD(1, 1), &wsa) != 0) {
        return 1;
    }
    if (!CreateCaptureSurface()) {
        return 1;
    }
    // Not every CE image exports mouse_event; mirror-only without it.
    coredll = LoadLibraryW(L"coredll.dll");
    if (coredll) {
        g_mouse_event = (MouseEventProc)GetProcAddress(coredll, L"mouse_event");
    }

    listener = socket(AF_INET, SOCK_STREAM, 0);
    if (listener == INVALID_SOCKET) {
        return 1;
    }
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons(RFB_PORT);
    if (bind(listener, (struct sockaddr *)&addr, sizeof(addr)) == SOCKET_ERROR ||
        listen(listener, 1) == SOCKET_ERROR) {
        closesocket(listener);
        return 1;
    }

    for (;;) {
        SOCKET client = accept(listener, NULL, NULL);
        if (client == INVALID_SOCKET) {
            break;
        }
        ServeClient(client);
        closesocket(client);
    }
    closesocket(listener);
    WSACleanup();
    if (mutex) {
        CloseHandle(mutex);
    }
    return 0;
}
