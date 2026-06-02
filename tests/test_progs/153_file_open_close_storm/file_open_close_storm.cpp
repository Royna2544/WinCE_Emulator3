#include <windows.h>
#include "../common/fixture_status.h"
#ifndef FILE_MAP_ALL_ACCESS
#define FILE_MAP_ALL_ACCESS 0x000f001f
#endif
int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    int i;
    for (i = 0; i < 150; ++i) {
        HANDLE f = CreateFileW(L"\\SDMMC Disk\\fixture_stress.tmp", GENERIC_READ | GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
        if (f == INVALID_HANDLE_VALUE) return FixtureFail(15301);
        DWORD written = 0; BYTE b = (BYTE)i; WriteFile(f, &b, 1, &written, 0);
        HANDLE m = CreateFileMappingW(f, 0, PAGE_READWRITE, 0, 4096, L"FixtureStressMap");
        if (m) {
            BYTE* v = (BYTE*)MapViewOfFile(m, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
            if (v) { v[0] = b; FlushViewOfFile(v, 1); UnmapViewOfFile(v); }
            CloseHandle(m);
        }
        CloseHandle(f);
        HKEY key = 0; DWORD disp = 0;
        if (RegCreateKeyExW(HKEY_CURRENT_USER, L"Software\\FixtureStress", 0, 0, 0, 0, 0, &key, &disp) == ERROR_SUCCESS) {
            RegSetValueExW(key, L"Value", 0, REG_DWORD, (const BYTE*)&i, sizeof(i));
            RegCloseKey(key);
        }
    }
    DeleteFileW(L"\\SDMMC Disk\\fixture_stress.tmp");
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\FixtureStress");
    return FIXTURE_OK;
}
