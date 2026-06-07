#include <windows.h>

BOOL WINAPI DllMain(HANDLE, DWORD, LPVOID) {
    return TRUE;
}

extern "C" DWORD WINAPI DependencyBaseValue(DWORD value) {
    return value + 0x5300;
}
