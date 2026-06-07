#include <windows.h>

static volatile DWORD g_attachCount = 0;

BOOL WINAPI DllMain(HANDLE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        ++g_attachCount;
    }
    return TRUE;
}

extern "C" DWORD WINAPI DependencyBaseValue(DWORD value) {
    return value + 0x5300;
}

extern "C" DWORD WINAPI DependencyBaseAttachCount() {
    return g_attachCount;
}
