#include <windows.h>

extern "C" __declspec(dllimport) DWORD WINAPI DependencyValue();

static volatile DWORD g_attachCount = 0;
static volatile DWORD g_detachCount = 0;

extern "C" DWORD WINAPI NoResolveAttachCount() {
    return g_attachCount;
}

extern "C" DWORD WINAPI NoResolveDetachCount() {
    return g_detachCount;
}

extern "C" DWORD WINAPI NoResolvePlainValue() {
    return 0x4455;
}

extern "C" DWORD WINAPI NoResolveImportedValue() {
    return DependencyValue();
}

BOOL WINAPI DllMain(HANDLE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        ++g_attachCount;
    } else if (reason == DLL_PROCESS_DETACH) {
        ++g_detachCount;
    }
    return TRUE;
}
