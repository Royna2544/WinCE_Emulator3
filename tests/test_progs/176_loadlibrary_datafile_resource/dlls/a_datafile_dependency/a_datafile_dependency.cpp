#include <windows.h>

extern "C" DWORD WINAPI DatafileDependencyValue() {
    return 0x176d;
}

BOOL WINAPI DllMain(HANDLE, DWORD, LPVOID) {
    return TRUE;
}
