// mini_ftp.cpp — tiny HTTP file server for Windows CE devices, port 5001.
//
//   GET  /list?path=\SDMMC Disk\TOOLS        -> text lines: "D <name>" or "F <size> <name>"
//   GET  /download?path=\ResidentFlash\x.bin -> raw file bytes (application/octet-stream)
//   POST /upload?path=\SDMMC Disk\x.exe      -> request body written verbatim (CREATE_ALWAYS)
//
// Paths are URL-decoded (%XX and '+'), must be absolute CE paths starting
// with '\'. Responses are HTTP/1.0 with Connection: close. One connection is
// served at a time; no authentication (trusted LAN tooling only).

#include <windows.h>
#include <winsock2.h>

#define SERVER_PORT 5001
#define REQUEST_MAX 4096
#define IO_CHUNK 16384
#define MAX_PATH_CE 260

// CE coredll exports only the wide string helpers, so provide the few ASCII
// operations this server needs over its all-ASCII HTTP text locally.
static int AsciiLen(const char *s) {
    int n = 0;
    while (s[n]) {
        ++n;
    }
    return n;
}

static void AsciiCopy(char *dst, const char *src) {
    while ((*dst++ = *src++) != 0) {
    }
}

// Append an unsigned decimal; returns the new end pointer.
static char *AppendUDec(char *out, DWORD value) {
    char tmp[12];
    int n = 0;
    if (value == 0) {
        *out++ = '0';
        return out;
    }
    while (value > 0) {
        tmp[n++] = (char)('0' + (value % 10));
        value /= 10;
    }
    while (n > 0) {
        *out++ = tmp[--n];
    }
    return out;
}

// Case-sensitive prefix test: does `text` begin with `prefix`?
static BOOL StartsWith(const char *text, const char *prefix) {
    while (*prefix) {
        if (*text++ != *prefix++) {
            return FALSE;
        }
    }
    return TRUE;
}

// Case-insensitive prefix test for header names.
static BOOL StartsWithNoCase(const char *text, const char *prefix) {
    while (*prefix) {
        char a = *text++;
        char b = *prefix++;
        if (a >= 'A' && a <= 'Z') {
            a = (char)(a - 'A' + 'a');
        }
        if (b >= 'A' && b <= 'Z') {
            b = (char)(b - 'A' + 'a');
        }
        if (a != b) {
            return FALSE;
        }
    }
    return TRUE;
}

static int RecvAll(SOCKET s, char *buffer, int len) {
    int total = 0;
    while (total < len) {
        int got = recv(s, buffer + total, len - total, 0);
        if (got <= 0) {
            return total;
        }
        total += got;
    }
    return total;
}

static BOOL SendAll(SOCKET s, const char *data, int len) {
    int total = 0;
    while (total < len) {
        int sent = send(s, data + total, len - total, 0);
        if (sent <= 0) {
            return FALSE;
        }
        total += sent;
    }
    return TRUE;
}

static BOOL SendText(SOCKET s, const char *text) {
    return SendAll(s, text, AsciiLen(text));
}

static void SendStatus(SOCKET s, const char *status, const char *body) {
    char header[256];
    char *p = header;
    AsciiCopy(p, "HTTP/1.0 ");
    p += AsciiLen(p);
    AsciiCopy(p, status);
    p += AsciiLen(p);
    AsciiCopy(p, "\r\nContent-Type: text/plain\r\nContent-Length: ");
    p += AsciiLen(p);
    p = AppendUDec(p, (DWORD)AsciiLen(body));
    AsciiCopy(p, "\r\nConnection: close\r\n\r\n");
    SendText(s, header);
    SendText(s, body);
}

static int HexValue(char c) {
    if (c >= '0' && c <= '9') {
        return c - '0';
    }
    if (c >= 'a' && c <= 'f') {
        return c - 'a' + 10;
    }
    if (c >= 'A' && c <= 'F') {
        return c - 'A' + 10;
    }
    return -1;
}

// URL-decode `src` into a wide CE path. Returns FALSE for malformed input.
static BOOL DecodePathParam(const char *src, WCHAR *dst, int dst_chars) {
    int out = 0;
    while (*src && out < dst_chars - 1) {
        char c = *src++;
        if (c == '+') {
            c = ' ';
        } else if (c == '%') {
            int hi = HexValue(src[0]);
            int lo = hi >= 0 ? HexValue(src[1]) : -1;
            if (lo < 0) {
                return FALSE;
            }
            c = (char)((hi << 4) | lo);
            src += 2;
        }
        dst[out++] = (WCHAR)(unsigned char)c;
    }
    dst[out] = 0;
    return out > 0 && dst[0] == L'\\';
}

// Extract the value of "path=" from a query string (up to '&' or end).
static BOOL QueryPath(const char *query, WCHAR *dst, int dst_chars) {
    char raw[REQUEST_MAX];
    int out = 0;
    const char *p = query;
    while (*p) {
        if (p[0] == 'p' && p[1] == 'a' && p[2] == 't' && p[3] == 'h' && p[4] == '=') {
            p += 5;
            while (*p && *p != '&' && *p != ' ' && out < (int)sizeof(raw) - 1) {
                raw[out++] = *p++;
            }
            raw[out] = 0;
            return DecodePathParam(raw, dst, dst_chars);
        }
        ++p;
    }
    return FALSE;
}

static void HandleList(SOCKET s, const WCHAR *path) {
    WCHAR pattern[MAX_PATH_CE + 4];
    WIN32_FIND_DATAW find;
    HANDLE handle;
    char line[600];
    char body[REQUEST_MAX * 4];
    int body_len = 0;

    wsprintfW(pattern, L"%s\\*", path);
    handle = FindFirstFileW(pattern, &find);
    if (handle == INVALID_HANDLE_VALUE) {
        SendStatus(s, "404 Not Found", "no such directory\n");
        return;
    }
    body[0] = 0;
    do {
        char name[MAX_PATH_CE];
        char *p = line;
        int line_len;
        WideCharToMultiByte(CP_ACP, 0, find.cFileName, -1, name, sizeof(name), NULL, NULL);
        if (find.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            AsciiCopy(p, "D ");
            p += AsciiLen(p);
            AsciiCopy(p, name);
            p += AsciiLen(p);
        } else {
            AsciiCopy(p, "F ");
            p += AsciiLen(p);
            p = AppendUDec(p, find.nFileSizeLow);
            *p++ = ' ';
            AsciiCopy(p, name);
            p += AsciiLen(p);
        }
        *p++ = '\n';
        *p = 0;
        line_len = AsciiLen(line);
        if (body_len + line_len < (int)sizeof(body) - 1) {
            AsciiCopy(body + body_len, line);
            body_len += line_len;
        }
    } while (FindNextFileW(handle, &find));
    FindClose(handle);
    SendStatus(s, "200 OK", body);
}

static void HandleDownload(SOCKET s, const WCHAR *path) {
    HANDLE file = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    char header[256];
    char chunk[IO_CHUNK];
    DWORD size;
    DWORD got;

    if (file == INVALID_HANDLE_VALUE) {
        SendStatus(s, "404 Not Found", "no such file\n");
        return;
    }
    size = GetFileSize(file, NULL);
    {
        char *p = header;
        AsciiCopy(p, "HTTP/1.0 200 OK\r\nContent-Type: application/octet-stream\r\n"
                     "Content-Length: ");
        p += AsciiLen(p);
        p = AppendUDec(p, size);
        AsciiCopy(p, "\r\nConnection: close\r\n\r\n");
    }
    if (!SendText(s, header)) {
        CloseHandle(file);
        return;
    }
    while (ReadFile(file, chunk, sizeof(chunk), &got, NULL) && got > 0) {
        if (!SendAll(s, chunk, (int)got)) {
            break;
        }
    }
    CloseHandle(file);
}

static void HandleUpload(SOCKET s, const WCHAR *path, const char *body_start, int body_have,
                         int content_length) {
    HANDLE file =
        CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    char chunk[IO_CHUNK];
    char reply[64];
    int remaining;
    DWORD wrote;

    if (file == INVALID_HANDLE_VALUE) {
        SendStatus(s, "500 Internal Server Error", "cannot create file\n");
        return;
    }
    if (body_have > 0) {
        WriteFile(file, body_start, (DWORD)body_have, &wrote, NULL);
    }
    remaining = content_length - body_have;
    while (remaining > 0) {
        int want = remaining < (int)sizeof(chunk) ? remaining : (int)sizeof(chunk);
        int got = RecvAll(s, chunk, want);
        if (got <= 0) {
            break;
        }
        WriteFile(file, chunk, (DWORD)got, &wrote, NULL);
        remaining -= got;
    }
    CloseHandle(file);
    if (remaining > 0) {
        SendStatus(s, "500 Internal Server Error", "short body\n");
    } else {
        char *p = reply;
        AsciiCopy(p, "OK ");
        p += AsciiLen(p);
        p = AppendUDec(p, (DWORD)content_length);
        AsciiCopy(p, " bytes\n");
        SendStatus(s, "200 OK", reply);
    }
}

static int FindHeaderEnd(const char *buffer, int len) {
    int i;
    for (i = 0; i + 3 < len; ++i) {
        if (buffer[i] == '\r' && buffer[i + 1] == '\n' && buffer[i + 2] == '\r' &&
            buffer[i + 3] == '\n') {
            return i + 4;
        }
    }
    return -1;
}

static int ParseContentLength(const char *headers) {
    const char *p = headers;
    while (*p) {
        if ((p[0] == 'C' || p[0] == 'c') && StartsWithNoCase(p, "Content-Length:")) {
            p += 15;
            while (*p == ' ') {
                ++p;
            }
            int value = 0;
            while (*p >= '0' && *p <= '9') {
                value = value * 10 + (*p - '0');
                ++p;
            }
            return value;
        }
        ++p;
    }
    return 0;
}

static void ServeConnection(SOCKET s) {
    char request[REQUEST_MAX];
    int have = 0;
    int header_end = -1;
    WCHAR path[MAX_PATH_CE];

    while (have < (int)sizeof(request) - 1) {
        int got = recv(s, request + have, (int)sizeof(request) - 1 - have, 0);
        if (got <= 0) {
            return;
        }
        have += got;
        request[have] = 0;
        header_end = FindHeaderEnd(request, have);
        if (header_end >= 0) {
            break;
        }
    }
    if (header_end < 0) {
        SendStatus(s, "400 Bad Request", "headers too large\n");
        return;
    }

    if (StartsWith(request, "GET /list?")) {
        if (QueryPath(request + 10, path, MAX_PATH_CE)) {
            HandleList(s, path);
        } else {
            SendStatus(s, "400 Bad Request", "bad path\n");
        }
    } else if (StartsWith(request, "GET /download?")) {
        if (QueryPath(request + 14, path, MAX_PATH_CE)) {
            HandleDownload(s, path);
        } else {
            SendStatus(s, "400 Bad Request", "bad path\n");
        }
    } else if (StartsWith(request, "POST /upload?")) {
        if (QueryPath(request + 13, path, MAX_PATH_CE)) {
            int content_length = ParseContentLength(request);
            HandleUpload(s, path, request + header_end, have - header_end, content_length);
        } else {
            SendStatus(s, "400 Bad Request", "bad path\n");
        }
    } else {
        SendStatus(s, "404 Not Found", "use /list /download /upload with ?path=\n");
    }
}

int WINAPI WinMain(HINSTANCE instance, HINSTANCE prev, LPWSTR cmd_line, int show) {
    WSADATA wsa;
    SOCKET listener;
    struct sockaddr_in addr;
    HANDLE mutex;

    (void)instance;
    (void)prev;
    (void)cmd_line;
    (void)show;

    mutex = CreateMutexW(NULL, FALSE, L"MiniFtpSingleton");
    if (mutex && GetLastError() == ERROR_ALREADY_EXISTS) {
        return 0;
    }
    if (WSAStartup(MAKEWORD(1, 1), &wsa) != 0) {
        return 1;
    }
    listener = socket(AF_INET, SOCK_STREAM, 0);
    if (listener == INVALID_SOCKET) {
        return 1;
    }
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons(SERVER_PORT);
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
        ServeConnection(client);
        shutdown(client, 1);
        closesocket(client);
    }
    closesocket(listener);
    WSACleanup();
    if (mutex) {
        CloseHandle(mutex);
    }
    return 0;
}
