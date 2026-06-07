#include <windows.h>

extern "C" __declspec(dllimport) DWORD WINAPI DatafileDependencyValue();

static volatile DWORD g_attachCount = 0;

extern "C" DWORD WINAPI DatafileAttachCount() {
    return g_attachCount;
}

extern "C" DWORD WINAPI DatafileImportedValue() {
    return DatafileDependencyValue();
}

BOOL WINAPI DllMain(HANDLE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        ++g_attachCount;
    }
    return TRUE;
}
