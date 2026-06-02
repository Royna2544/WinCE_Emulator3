#include <windows.h>

#ifndef WM_APP
#define WM_APP 0x8000
#endif

#ifndef INVALID_SET_FILE_POINTER
#define INVALID_SET_FILE_POINTER ((DWORD)-1)
#endif

#define FIXTURE_OK 0
#define FIXTURE_FAIL_BASE 0x1000

static DWORD Fail(DWORD code) { return FIXTURE_FAIL_BASE + code; }

static int WideEqualsAscii(const wchar_t* wide, const char* ascii) {
    int i = 0;
    while (wide[i] && ascii[i]) {
        if (wide[i] != (wchar_t)(unsigned char)ascii[i]) return 0;
        ++i;
    }
    return wide[i] == 0 && ascii[i] == 0;
}

static int BytesEqual(const BYTE* a, const BYTE* b, DWORD count) {
    for (DWORD i = 0; i < count; ++i) {
        if (a[i] != b[i]) return 0;
    }
    return 1;
}

static DWORD PhaseSystemAndTime() {
    SYSTEM_INFO si;
    ZeroMemory(&si, sizeof(si));
    GetSystemInfo(&si);
    if (si.dwPageSize == 0) return Fail(101);

    MEMORYSTATUS ms;
    ZeroMemory(&ms, sizeof(ms));
    ms.dwLength = sizeof(ms);
    GlobalMemoryStatus(&ms);
    if (ms.dwLength != sizeof(ms)) return Fail(102);

    OSVERSIONINFO ov;
    ZeroMemory(&ov, sizeof(ov));
    ov.dwOSVersionInfoSize = sizeof(ov);
    if (!GetVersionEx(&ov)) return Fail(103);

    LARGE_INTEGER freq, c0, c1;
    if (!QueryPerformanceFrequency(&freq)) return Fail(104);
    if (freq.QuadPart == 0) return Fail(105);
    if (!QueryPerformanceCounter(&c0)) return Fail(106);

    DWORD t0 = GetTickCount();
    Sleep(1);
    DWORD t1 = GetTickCount();

    if (!QueryPerformanceCounter(&c1)) return Fail(107);
    if (t1 < t0) return Fail(108);
    if (c1.QuadPart < c0.QuadPart) return Fail(109);

    SetLastError(0x1234);
    if (GetLastError() != 0x1234) return Fail(110);

    return FIXTURE_OK;
}

static DWORD PhaseMemoryAndRects() {
    BYTE* local = (BYTE*)LocalAlloc(LPTR, 64);
    if (!local) return Fail(201);

    for (DWORD i = 0; i < 64; ++i) local[i] = (BYTE)(i ^ 0x5a);

    if (IsBadReadPtr(local, 64)) {
        LocalFree(local);
        return Fail(202);
    }
    if (IsBadWritePtr(local, 64)) {
        LocalFree(local);
        return Fail(203);
    }

    HLOCAL resized = LocalReAlloc((HLOCAL)local, 128, LMEM_MOVEABLE | LMEM_ZEROINIT);
    if (!resized) {
        LocalFree(local);
        return Fail(204);
    }
    local = (BYTE*)resized;

    HANDLE heap = GetProcessHeap();
    if (!heap) {
        LocalFree(local);
        return Fail(205);
    }

    BYTE* heapMem = (BYTE*)HeapAlloc(heap, HEAP_ZERO_MEMORY, 96);
    if (!heapMem) {
        LocalFree(local);
        return Fail(206);
    }

    heapMem[0] = 0x11;
    heapMem[95] = 0x22;

    BYTE* heapMem2 = (BYTE*)HeapReAlloc(heap, HEAP_ZERO_MEMORY, heapMem, 160);
    if (!heapMem2) {
        HeapFree(heap, 0, heapMem);
        LocalFree(local);
        return Fail(207);
    }
    heapMem = heapMem2;

    if (heapMem[0] != 0x11 || heapMem[95] != 0x22) {
        HeapFree(heap, 0, heapMem);
        LocalFree(local);
        return Fail(208);
    }

    void* virt = VirtualAlloc(0, 4096, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (!virt) {
        HeapFree(heap, 0, heapMem);
        LocalFree(local);
        return Fail(209);
    }

    ((BYTE*)virt)[0] = 0x33;
    ((BYTE*)virt)[4095] = 0x44;

    if (((BYTE*)virt)[0] != 0x33 || ((BYTE*)virt)[4095] != 0x44) {
        VirtualFree(virt, 0, MEM_RELEASE);
        HeapFree(heap, 0, heapMem);
        LocalFree(local);
        return Fail(210);
    }

    RECT r, copy, empty;
    POINT pt;
    pt.x = 10;
    pt.y = 20;

    SetRect(&r, 10, 20, 30, 50);
    CopyRect(&copy, &r);
    if (!EqualRect(&r, &copy)) {
        VirtualFree(virt, 0, MEM_RELEASE);
        HeapFree(heap, 0, heapMem);
        LocalFree(local);
        return Fail(211);
    }

    InflateRect(&copy, 5, 5);
    if (!PtInRect(&copy, pt)) {
        VirtualFree(virt, 0, MEM_RELEASE);
        HeapFree(heap, 0, heapMem);
        LocalFree(local);
        return Fail(212);
    }

    SetRectEmpty(&empty);
    if (!IsRectEmpty(&empty)) {
        VirtualFree(virt, 0, MEM_RELEASE);
        HeapFree(heap, 0, heapMem);
        LocalFree(local);
        return Fail(213);
    }

    VirtualFree(virt, 0, MEM_RELEASE);
    HeapFree(heap, 0, heapMem);
    LocalFree(local);

    return FIXTURE_OK;
}

static DWORD PhaseStringConversion() {
    const char* narrow = "api-storm";
    wchar_t wide[64];
    char roundTrip[64];

    ZeroMemory(wide, sizeof(wide));
    ZeroMemory(roundTrip, sizeof(roundTrip));

    int wideLen = MultiByteToWideChar(CP_ACP, 0, narrow, -1, wide, 64);
    if (wideLen <= 0) return Fail(301);
    if (!WideEqualsAscii(wide, narrow)) return Fail(302);

    int narrowLen = WideCharToMultiByte(CP_ACP, 0, wide, -1, roundTrip, 64, 0, 0);
    if (narrowLen <= 0) return Fail(303);

    for (int i = 0; narrow[i]; ++i) {
        if (roundTrip[i] != narrow[i]) return Fail(304);
    }

    wchar_t lower[] = L"abc";
    wchar_t upper[] = L"XYZ";
    CharUpperW(lower);
    CharLowerW(upper);

    if (lower[0] != L'A' || lower[1] != L'B' || lower[2] != L'C') return Fail(305);
    if (upper[0] != L'x' || upper[1] != L'y' || upper[2] != L'z') return Fail(306);

    return FIXTURE_OK;
}

static DWORD PhaseFileIo() {
    const wchar_t* dir = L"\\SDMMC Disk\\fixture_api_storm";
    const wchar_t* path = L"\\SDMMC Disk\\fixture_api_storm\\storm.tmp";
    const BYTE payload[] = {0x10, 0x20, 0x30, 0x40, 0x41, 0x50, 0x49, 0x21};
    BYTE readBack[sizeof(payload)];

    CreateDirectoryW(dir, 0);

    HANDLE file = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (file == INVALID_HANDLE_VALUE) return Fail(401);

    DWORD written = 0;
    if (!WriteFile(file, payload, sizeof(payload), &written, 0)) {
        CloseHandle(file);
        return Fail(402);
    }
    if (written != sizeof(payload)) {
        CloseHandle(file);
        return Fail(403);
    }

    if (!FlushFileBuffers(file)) {
        CloseHandle(file);
        return Fail(404);
    }

    DWORD size = GetFileSize(file, 0);
    if (size != sizeof(payload)) {
        CloseHandle(file);
        return Fail(405);
    }

    DWORD pos = SetFilePointer(file, 0, 0, FILE_BEGIN);
    if (pos == INVALID_SET_FILE_POINTER && GetLastError() != NO_ERROR) {
        CloseHandle(file);
        return Fail(406);
    }

    ZeroMemory(readBack, sizeof(readBack));
    DWORD read = 0;
    if (!ReadFile(file, readBack, sizeof(readBack), &read, 0)) {
        CloseHandle(file);
        return Fail(407);
    }
    if (read != sizeof(payload)) {
        CloseHandle(file);
        return Fail(408);
    }
    if (!BytesEqual(readBack, payload, sizeof(payload))) {
        CloseHandle(file);
        return Fail(409);
    }

    CloseHandle(file);

    DWORD attrs = GetFileAttributesW(path);
    if (attrs == 0xffffffff) return Fail(410);

    WIN32_FIND_DATAW findData;
    ZeroMemory(&findData, sizeof(findData));

    HANDLE find = FindFirstFileW(L"\\SDMMC Disk\\fixture_api_storm\\*.tmp", &findData);
    if (find == INVALID_HANDLE_VALUE) return Fail(411);

    int found = 0;
    do {
        if (findData.cFileName[0]) {
            found = 1;
            break;
        }
    } while (FindNextFileW(find, &findData));

    FindClose(find);

    if (!found) return Fail(412);

    DeleteFileW(path);
    return FIXTURE_OK;
}

static DWORD PhaseRegistry() {
    HKEY key = 0;
    DWORD disposition = 0;
    LONG rc = RegCreateKeyExW(HKEY_CURRENT_USER, L"Software\\FixtureApiStorm", 0, 0, 0, 0, 0, &key, &disposition);
    if (rc != ERROR_SUCCESS) return Fail(501);

    DWORD value = 0x55667788;
    rc = RegSetValueExW(key, L"Number", 0, REG_DWORD, (const BYTE*)&value, sizeof(value));
    if (rc != ERROR_SUCCESS) {
        RegCloseKey(key);
        return Fail(502);
    }

    const wchar_t text[] = L"registry-ok";
    rc = RegSetValueExW(key, L"Text", 0, REG_SZ, (const BYTE*)text, sizeof(text));
    if (rc != ERROR_SUCCESS) {
        RegCloseKey(key);
        return Fail(503);
    }

    DWORD type = 0;
    DWORD readValue = 0;
    DWORD cb = sizeof(readValue);
    rc = RegQueryValueExW(key, L"Number", 0, &type, (BYTE*)&readValue, &cb);
    if (rc != ERROR_SUCCESS || type != REG_DWORD || readValue != value) {
        RegCloseKey(key);
        return Fail(504);
    }

    wchar_t name[64];
    BYTE data[128];
    DWORD nameChars = 64;
    DWORD dataBytes = sizeof(data);
    type = 0;

    rc = RegEnumValueW(key, 0, name, &nameChars, 0, &type, data, &dataBytes);
    if (rc != ERROR_SUCCESS) {
        RegCloseKey(key);
        return Fail(505);
    }

    RegDeleteValueW(key, L"Number");
    RegDeleteValueW(key, L"Text");
    RegCloseKey(key);
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\FixtureApiStorm");

    return FIXTURE_OK;
}

struct ThreadArgs {
    HANDLE eventHandle;
    DWORD tlsSlot;
    DWORD observedInitial;
    DWORD workerValue;
};

static DWORD WINAPI WorkerThreadProc(LPVOID param) {
    ThreadArgs* args = (ThreadArgs*)param;
    args->observedInitial = (DWORD)TlsGetValue(args->tlsSlot);
    TlsSetValue(args->tlsSlot, (LPVOID)0x22224444);
    args->workerValue = (DWORD)TlsGetValue(args->tlsSlot);
    SetEvent(args->eventHandle);
    return 0;
}

static DWORD PhaseThreadSyncTls() {
    CRITICAL_SECTION cs;
    InitializeCriticalSection(&cs);
    EnterCriticalSection(&cs);
    LeaveCriticalSection(&cs);
    DeleteCriticalSection(&cs);

    DWORD slot = TlsAlloc();
    if (slot == TLS_OUT_OF_INDEXES) return Fail(601);

    if (!TlsSetValue(slot, (LPVOID)0x11113333)) {
        TlsFree(slot);
        return Fail(602);
    }
    if ((DWORD)TlsGetValue(slot) != 0x11113333) {
        TlsFree(slot);
        return Fail(603);
    }

    HANDLE eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!eventHandle) {
        TlsFree(slot);
        return Fail(604);
    }

    ThreadArgs args;
    args.eventHandle = eventHandle;
    args.tlsSlot = slot;
    args.observedInitial = 0xffffffff;
    args.workerValue = 0xffffffff;

    DWORD threadId = 0;
    HANDLE thread = CreateThread(0, 0, WorkerThreadProc, &args, 0, &threadId);
    if (!thread) {
        CloseHandle(eventHandle);
        TlsFree(slot);
        return Fail(605);
    }

    DWORD wait = WaitForSingleObject(eventHandle, 5000);
    if (wait != WAIT_OBJECT_0) {
        CloseHandle(thread);
        CloseHandle(eventHandle);
        TlsFree(slot);
        return Fail(606);
    }

    if ((DWORD)TlsGetValue(slot) != 0x11113333) {
        CloseHandle(thread);
        CloseHandle(eventHandle);
        TlsFree(slot);
        return Fail(607);
    }

    if (args.observedInitial != 0) {
        CloseHandle(thread);
        CloseHandle(eventHandle);
        TlsFree(slot);
        return Fail(608);
    }

    if (args.workerValue != 0x22224444) {
        CloseHandle(thread);
        CloseHandle(eventHandle);
        TlsFree(slot);
        return Fail(609);
    }

    WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    CloseHandle(eventHandle);
    TlsFree(slot);

    return FIXTURE_OK;
}

static DWORD g_painted = 0;
static DWORD g_sentMessages = 0;
static DWORD g_postedMessages = 0;

static LRESULT CALLBACK StormWndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
    case WM_CREATE:
        InvalidateRect(hwnd, 0, TRUE);
        return 0;

    case WM_PAINT: {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(hwnd, &ps);
        if (dc) {
            RECT client;
            GetClientRect(hwnd, &client);

            HBRUSH brush = CreateSolidBrush(RGB(16, 48, 96));
            FillRect(dc, &client, brush);
            DeleteObject(brush);

            HPEN pen = CreatePen(PS_SOLID, 2, RGB(255, 255, 0));
            HBRUSH ellipseBrush = CreateSolidBrush(RGB(200, 32, 32));
            HPEN oldPen = (HPEN)SelectObject(dc, pen);
            HBRUSH oldBrush = (HBRUSH)SelectObject(dc, ellipseBrush);

            Rectangle(dc, 10, 10, 110, 60);
            Ellipse(dc, 20, 20, 100, 70);
            MoveToEx(dc, 0, 0, 0);
            LineTo(dc, 119, 79);

            SelectObject(dc, oldPen);
            SelectObject(dc, oldBrush);
            DeleteObject(pen);
            DeleteObject(ellipseBrush);

            HDC memdc = CreateCompatibleDC(dc);
            HBITMAP bitmap = CreateCompatibleBitmap(dc, 16, 16);
            if (memdc && bitmap) {
                HBITMAP oldBitmap = (HBITMAP)SelectObject(memdc, bitmap);
                RECT small;
                SetRect(&small, 0, 0, 16, 16);
                HBRUSH smallBrush = CreateSolidBrush(RGB(0, 200, 0));
                FillRect(memdc, &small, smallBrush);
                DeleteObject(smallBrush);
                BitBlt(dc, 2, 2, 16, 16, memdc, 0, 0, SRCCOPY);
                SelectObject(memdc, oldBitmap);
            }
            if (bitmap) DeleteObject(bitmap);
            if (memdc) DeleteDC(memdc);

            g_painted = 1;
        }
        EndPaint(hwnd, &ps);
        return 0;
    }

    case WM_USER + 1:
        g_sentMessages += (DWORD)wParam;
        return 0x2468;

    case WM_USER + 2:
        g_postedMessages += (DWORD)wParam;
        return 0;

    case WM_DESTROY:
        return 0;
    }

    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

static DWORD PhaseWindowGdiMessages(HINSTANCE hInstance) {
    g_painted = 0;
    g_sentMessages = 0;
    g_postedMessages = 0;

    const wchar_t* className = L"ApiStormWindowClass";

    WNDCLASSW wc;
    ZeroMemory(&wc, sizeof(wc));
    wc.lpfnWndProc = StormWndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = className;

    if (!RegisterClassW(&wc)) return Fail(701);

    HWND hwnd = CreateWindowExW(0, className, L"api storm", WS_VISIBLE, 0, 0, 120, 80, 0, 0, hInstance, 0);
    if (!hwnd) return Fail(702);

    wchar_t title[64];
    ZeroMemory(title, sizeof(title));

    if (!SetWindowTextW(hwnd, L"api storm title")) {
        DestroyWindow(hwnd);
        return Fail(703);
    }

    int titleLen = GetWindowTextLengthW(hwnd);
    if (titleLen <= 0) {
        DestroyWindow(hwnd);
        return Fail(704);
    }

    GetWindowTextW(hwnd, title, 64);
    if (!WideEqualsAscii(title, "api storm title")) {
        DestroyWindow(hwnd);
        return Fail(705);
    }

    ShowWindow(hwnd, SW_SHOW);
    MoveWindow(hwnd, 5, 7, 120, 80, TRUE);

    HDC dc = GetDC(hwnd);
    if (!dc) {
        DestroyWindow(hwnd);
        return Fail(706);
    }

    int width = GetDeviceCaps(dc, HORZRES);
    int height = GetDeviceCaps(dc, VERTRES);
    ReleaseDC(hwnd, dc);

    if (width <= 0 || height <= 0) {
        DestroyWindow(hwnd);
        return Fail(707);
    }

    SetFocus(hwnd);
    SetCapture(hwnd);
    if (GetCapture() != hwnd) {
        ReleaseCapture();
        DestroyWindow(hwnd);
        return Fail(708);
    }
    ReleaseCapture();

    LRESULT sendResult = SendMessageW(hwnd, WM_USER + 1, 3, 0);
    if (sendResult != 0x2468 || g_sentMessages != 3) {
        DestroyWindow(hwnd);
        return Fail(709);
    }

    if (!PostMessageW(hwnd, WM_USER + 2, 5, 0)) {
        DestroyWindow(hwnd);
        return Fail(710);
    }

    MSG msg;
    DWORD spins = 0;
    while (g_postedMessages != 5 && spins < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
        ++spins;
    }

    if (g_postedMessages != 5) {
        DestroyWindow(hwnd);
        return Fail(711);
    }

    InvalidateRect(hwnd, 0, TRUE);
    UpdateWindow(hwnd);

    spins = 0;
    while (!g_painted && spins < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
        ++spins;
    }

    if (!g_painted) {
        DestroyWindow(hwnd);
        return Fail(712);
    }

    DestroyWindow(hwnd);
    return FIXTURE_OK;
}

static DWORD PhaseRepeatPressure() {
    for (DWORD i = 0; i < 10; ++i) {
        DWORD rc = PhaseMemoryAndRects();
        if (rc != FIXTURE_OK) return Fail(800 + i);

        rc = PhaseFileIo();
        if (rc != FIXTURE_OK) return Fail(820 + i);

        rc = PhaseSystemAndTime();
        if (rc != FIXTURE_OK) return Fail(840 + i);
    }
    return FIXTURE_OK;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int) {
    DWORD rc = FIXTURE_OK;

    rc = PhaseSystemAndTime();
    if (rc != FIXTURE_OK) return rc;

    rc = PhaseMemoryAndRects();
    if (rc != FIXTURE_OK) return rc;

    rc = PhaseStringConversion();
    if (rc != FIXTURE_OK) return rc;

    rc = PhaseFileIo();
    if (rc != FIXTURE_OK) return rc;

    rc = PhaseRegistry();
    if (rc != FIXTURE_OK) return rc;

    rc = PhaseThreadSyncTls();
    if (rc != FIXTURE_OK) return rc;

    rc = PhaseWindowGdiMessages(hInstance);
    if (rc != FIXTURE_OK) return rc;

    rc = PhaseRepeatPressure();
    if (rc != FIXTURE_OK) return rc;

    return FIXTURE_OK;
}
