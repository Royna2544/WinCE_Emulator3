#include <windows.h>

extern "C" DWORD WINAPI DependencyValue() {
    return 0x7788;
}

BOOL WINAPI DllMain(HANDLE, DWORD, LPVOID) {
    return TRUE;
}
