#include <windows.h>

static volatile DWORD g_attachCount = 0;
static volatile DWORD g_detachCount = 0;

extern "C" BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        ++g_attachCount;
    } else if (reason == DLL_PROCESS_DETACH) {
        ++g_detachCount;
    }
    return TRUE;
}

extern "C" DWORD WINAPI FixtureNamedExport(DWORD value) {
    return value + 0x3434;
}

extern "C" DWORD WINAPI FixtureOrdinalExport(DWORD value) {
    return value + 0x7047;
}

extern "C" DWORD WINAPI FixtureAttachCount() {
    return g_attachCount;
}

extern "C" DWORD WINAPI FixtureDetachCount() {
    return g_detachCount;
}
