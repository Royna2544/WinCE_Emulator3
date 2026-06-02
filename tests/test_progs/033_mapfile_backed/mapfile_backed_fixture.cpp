#include <windows.h>
#include "../common/fixture_status.h"

#ifndef FILE_MAP_ALL_ACCESS
#define FILE_MAP_ALL_ACCESS 0x000f001f
#endif
#ifndef INVALID_SET_FILE_POINTER
#define INVALID_SET_FILE_POINTER ((DWORD)-1)
#endif

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    const wchar_t* dir = L"\\SDMMC Disk\\fixture_mapfile";
    const wchar_t* path = L"\\SDMMC Disk\\fixture_mapfile\\mapped.bin";

    CreateDirectoryW(dir, 0);

    HANDLE file = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (file == INVALID_HANDLE_VALUE) return FixtureFail(2501);

    BYTE init[256];
    DWORD i;
    for (i = 0; i < sizeof(init); ++i) init[i] = (BYTE)(i ^ 0x3c);

    DWORD written = 0;
    if (!WriteFile(file, init, sizeof(init), &written, 0) || written != sizeof(init)) return FixtureFail(2502);

    HANDLE map = CreateFileMappingW(file, 0, PAGE_READWRITE, 0, 4096, L"FixtureFileBackedMapW");
    if (!map) return FixtureFail(2503);

    BYTE* view = (BYTE*)MapViewOfFile(map, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
    if (!view) return FixtureFail(2504);

    if (!BytesEq(view, init, sizeof(init))) return FixtureFail(2505);

    view[0] = 'M';
    view[1] = 'A';
    view[2] = 'P';
    view[255] = 0x7e;

    if (!FlushViewOfFile(view, 256)) return FixtureFail(2506);

    UnmapViewOfFile(view);
    CloseHandle(map);

    DWORD pos = SetFilePointer(file, 0, 0, FILE_BEGIN);
    if (pos == INVALID_SET_FILE_POINTER && GetLastError() != NO_ERROR) return FixtureFail(2507);

    BYTE readBack[256];
    DWORD read = 0;
    if (!ReadFile(file, readBack, sizeof(readBack), &read, 0) || read != sizeof(readBack)) return FixtureFail(2508);

    if (readBack[0] != 'M' || readBack[1] != 'A' || readBack[2] != 'P' || readBack[255] != 0x7e) return FixtureFail(2509);

    CloseHandle(file);
    DeleteFileW(path);
    return FIXTURE_OK;
}
