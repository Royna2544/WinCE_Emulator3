#include <windows.h>

extern "C" __declspec(dllimport) DWORD WINAPI DependencyBaseValue(DWORD value);

static volatile DWORD g_attachCount = 0;

BOOL WINAPI DllMain(HANDLE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        ++g_attachCount;
    }
    return TRUE;
}

extern "C" DWORD WINAPI DependentUserExport(DWORD value) {
    return DependencyBaseValue(value + 0x35);
}

extern "C" DWORD WINAPI DependencyUserAttachCount() {
    return g_attachCount;
}
