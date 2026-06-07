#include <windows.h>

extern "C" __declspec(dllimport) DWORD WINAPI DependencyBaseValue(DWORD value);

BOOL WINAPI DllMain(HANDLE, DWORD, LPVOID) {
    return TRUE;
}

extern "C" DWORD WINAPI DependentUserExport(DWORD value) {
    return DependencyBaseValue(value + 0x35);
}
