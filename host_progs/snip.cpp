#include <windows.h>

#pragma comment(lib, "coredll.lib")

#define MAX_PATH_CE 260
#define SNIP_FROM_YEAR 2005
#define SNIP_FROM_MONTH 4
#define SNIP_FROM_DAY 4
#define DEFAULT_SNIP_COPY_DIR L"\\SDMMC Disk"

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

static const WCHAR *SkipSpaces(const WCHAR *text) {
    while (text && (*text == L' ' || *text == L'\t' || *text == L'\r' || *text == L'\n')) {
        ++text;
    }
    return text;
}

static DWORD SecondsSinceSnipDate() {
    SYSTEMTIME start_st;
    SYSTEMTIME now_st;
    FILETIME start_ft;
    FILETIME now_ft;
    ULARGE_INTEGER start;
    ULARGE_INTEGER now;

    ZeroMemory(&start_st, sizeof(start_st));
    start_st.wYear = SNIP_FROM_YEAR;
    start_st.wMonth = SNIP_FROM_MONTH;
    start_st.wDay = SNIP_FROM_DAY;
    start_st.wHour = 0;
    start_st.wMinute = 0;
    start_st.wSecond = 0;

    GetLocalTime(&now_st);
    if (!SystemTimeToFileTime(&start_st, &start_ft) || !SystemTimeToFileTime(&now_st, &now_ft)) {
        return GetTickCount();
    }
    start.LowPart = start_ft.dwLowDateTime;
    start.HighPart = start_ft.dwHighDateTime;
    now.LowPart = now_ft.dwLowDateTime;
    now.HighPart = now_ft.dwHighDateTime;
    if (now.QuadPart <= start.QuadPart) {
        return 0;
    }
    return (DWORD)((now.QuadPart - start.QuadPart) / (ULONGLONG)10000000);
}

static void BuildOutputPath(const WCHAR *dir_arg, WCHAR *out, int out_chars) {
    const WCHAR *dir = SkipSpaces(dir_arg);
    if (!dir || !*dir) {
        dir = DEFAULT_SNIP_COPY_DIR;
    }
    CopyString(out, out_chars, dir);
    int len = lstrlenW(out);
    if (len > 0 && out[len - 1] != L'\\') {
        CatString(out, out_chars, L"\\");
    }
    WCHAR name[80];
    wsprintfW(name, L"SnipTool_%lu.bmp", SecondsSinceSnipDate());
    CatString(out, out_chars, name);
}

static BOOL SaveBitmapFile(LPCWSTR path, const BYTE *pixels, int width, int height, DWORD image_bytes) {
    BITMAPINFOHEADER bih;
    BITMAPFILEHEADER bfh;
    HANDLE file = INVALID_HANDLE_VALUE;
    DWORD written = 0;
    BOOL ok = FALSE;

    ZeroMemory(&bih, sizeof(bih));
    bih.biSize = sizeof(bih);
    bih.biWidth = width;
    bih.biHeight = height;
    bih.biPlanes = 1;
    bih.biBitCount = 24;
    bih.biCompression = BI_RGB;
    bih.biSizeImage = image_bytes;

    ZeroMemory(&bfh, sizeof(bfh));
    bfh.bfType = 0x4d42;
    bfh.bfOffBits = sizeof(bfh) + sizeof(bih);
    bfh.bfSize = bfh.bfOffBits + image_bytes;

    file = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (file == INVALID_HANDLE_VALUE) {
        goto done;
    }
    if (!WriteFile(file, &bfh, sizeof(bfh), &written, NULL) || written != sizeof(bfh)) {
        goto done;
    }
    if (!WriteFile(file, &bih, sizeof(bih), &written, NULL) || written != sizeof(bih)) {
        goto done;
    }
    if (!WriteFile(file, pixels, image_bytes, &written, NULL) || written != image_bytes) {
        goto done;
    }
    ok = TRUE;

done:
    if (file != INVALID_HANDLE_VALUE) {
        CloseHandle(file);
    }
    return ok;
}

static BOOL CaptureScreen(LPCWSTR path) {
    int width = GetSystemMetrics(SM_CXSCREEN);
    int height = GetSystemMetrics(SM_CYSCREEN);
    HDC screen_dc = GetDC(NULL);
    HDC mem_dc = 0;
    HBITMAP bitmap = 0;
    HGDIOBJ old_bitmap = 0;
    BITMAPINFO bmi;
    BYTE *pixels = 0;
    DWORD stride = ((DWORD)width * 3 + 3) & ~3;
    DWORD image_bytes = stride * (DWORD)height;
    BOOL ok = FALSE;

    if (!screen_dc) {
        return FALSE;
    }
    mem_dc = CreateCompatibleDC(screen_dc);
    if (!mem_dc) {
        goto done;
    }
    ZeroMemory(&bmi, sizeof(bmi));
    bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = height;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 24;
    bmi.bmiHeader.biCompression = BI_RGB;
    bmi.bmiHeader.biSizeImage = image_bytes;
    bitmap = CreateDIBSection(screen_dc, &bmi, DIB_RGB_COLORS, (VOID **)&pixels, NULL, 0);
    if (!bitmap) {
        goto done;
    }
    old_bitmap = SelectObject(mem_dc, bitmap);
    if (!BitBlt(mem_dc, 0, 0, width, height, screen_dc, 0, 0, SRCCOPY)) {
        goto done;
    }
    ok = SaveBitmapFile(path, pixels, width, height, image_bytes);

done:
    if (old_bitmap) {
        SelectObject(mem_dc, old_bitmap);
    }
    if (bitmap) {
        DeleteObject(bitmap);
    }
    if (mem_dc) {
        DeleteDC(mem_dc);
    }
    ReleaseDC(NULL, screen_dc);
    return ok;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR command_line, int) {
    WCHAR path[MAX_PATH_CE];
    WCHAR message[MAX_PATH_CE + 80];
    BuildOutputPath(command_line, path, MAX_PATH_CE);
    if (CaptureScreen(path)) {
        CopyString(message, MAX_PATH_CE + 80, L"Screenshot saved:\r\n");
        CatString(message, MAX_PATH_CE + 80, path);
        MessageBoxW(NULL, message, L"Snip", MB_OK | MB_ICONINFORMATION);
        return 0;
    }
    wsprintfW(message, L"Screenshot failed.\r\nGetLastError=%lu", GetLastError());
    MessageBoxW(NULL, message, L"Snip", MB_OK | MB_ICONERROR);
    return 1;
}
