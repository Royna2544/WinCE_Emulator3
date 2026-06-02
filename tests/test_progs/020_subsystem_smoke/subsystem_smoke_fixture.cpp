#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    SYSTEM_INFO systemInfo;
    ZeroMemory(&systemInfo, sizeof(systemInfo));
    GetSystemInfo(&systemInfo);
    if (systemInfo.dwPageSize == 0) {
        return FixtureFail(1);
    }

    MEMORYSTATUS memoryStatus;
    ZeroMemory(&memoryStatus, sizeof(memoryStatus));
    memoryStatus.dwLength = sizeof(memoryStatus);
    GlobalMemoryStatus(&memoryStatus);
    if (memoryStatus.dwLength != sizeof(memoryStatus)) {
        return FixtureFail(2);
    }

    void* virtualBlock = VirtualAlloc(0, 4096, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (!virtualBlock) {
        return FixtureFail(3);
    }
    ((DWORD*)virtualBlock)[0] = 0x12345678;
    if (((DWORD*)virtualBlock)[0] != 0x12345678) {
        return FixtureFail(4);
    }
    if (!VirtualFree(virtualBlock, 0, MEM_RELEASE)) {
        return FixtureFail(5);
    }

    HLOCAL localBlock = LocalAlloc(LMEM_FIXED, 32);
    if (!localBlock) {
        return FixtureFail(6);
    }
    ((BYTE*)localBlock)[0] = 0x5a;
    if (LocalFree(localBlock) != 0) {
        return FixtureFail(7);
    }

    HANDLE heap = GetProcessHeap();
    if (!heap) {
        return FixtureFail(8);
    }
    void* heapBlock = HeapAlloc(heap, 0, 64);
    if (!heapBlock) {
        return FixtureFail(9);
    }
    ((BYTE*)heapBlock)[0] = 0xa5;
    if (!HeapFree(heap, 0, heapBlock)) {
        return FixtureFail(10);
    }

    HKEY key = 0;
    LONG openResult = RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"ControlPanel", 0, 0, &key);
    if (openResult != ERROR_SUCCESS || !key) {
        return FixtureFail(11);
    }

    DWORD type = 0;
    DWORD value = 0;
    DWORD size = sizeof(value);
    LONG queryResult = RegQueryValueExW(key, L"InputConfig", 0, &type, (LPBYTE)&value, &size);
    RegCloseKey(key);
    if (queryResult != ERROR_SUCCESS) {
        return FixtureFail(12);
    }
    if (type != REG_DWORD || size != sizeof(value)) {
        return FixtureFail(13);
    }

    return FIXTURE_OK;
}
