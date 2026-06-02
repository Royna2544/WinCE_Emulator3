#include <windows.h>
#include "../common/fixture_status.h"

#ifndef FILE_MAP_ALL_ACCESS
#define FILE_MAP_ALL_ACCESS 0x000f001f
#endif

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    const wchar_t* name = L"FixtureMappingSameProcessW";
    HANDLE m = CreateFileMappingW(INVALID_HANDLE_VALUE, 0, PAGE_READWRITE, 0, 4096, name);
    if (!m) return FixtureFail(2401);

    BYTE* a = (BYTE*)MapViewOfFile(m, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
    if (!a) return FixtureFail(2402);

    BYTE payload[] = { 'M', 'A', 'P', 'W', 1, 2, 3, 4 };
    DWORD i;
    for (i = 0; i < sizeof(payload); ++i) a[i] = payload[i];
    if (!FlushViewOfFile(a, sizeof(payload))) return FixtureFail(2403);

    BYTE* b = (BYTE*)MapViewOfFile(m, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
    if (!b) return FixtureFail(2404);
    if (!BytesEq(b, payload, sizeof(payload))) return FixtureFail(2405);

    b[16] = 0x5a;
    if (a[16] != 0x5a) return FixtureFail(2406);

    UnmapViewOfFile(b);
    UnmapViewOfFile(a);
    CloseHandle(m);
    return FIXTURE_OK;
}
