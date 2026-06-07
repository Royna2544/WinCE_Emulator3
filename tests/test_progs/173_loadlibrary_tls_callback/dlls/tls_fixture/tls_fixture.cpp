#include <windows.h>

typedef VOID (NTAPI *PFN_TLS_CALLBACK)(PVOID, DWORD, PVOID);

struct IMAGE_TLS_DIRECTORY32_FIXTURE {
    DWORD StartAddressOfRawData;
    DWORD EndAddressOfRawData;
    DWORD AddressOfIndex;
    DWORD AddressOfCallBacks;
    DWORD SizeOfZeroFill;
    DWORD Characteristics;
};

static volatile DWORD g_tlsCount = 0;
static volatile DWORD g_attachCount = 0;
static volatile DWORD g_orderWord = 0;
static volatile DWORD g_nextOrder = 1;
static volatile DWORD *g_detachOrderObserved = 0;

void NTAPI FixtureTlsCallback(PVOID, DWORD reason, PVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        ++g_tlsCount;
        g_orderWord = (g_orderWord << 8) | g_nextOrder++;
    } else if (reason == DLL_PROCESS_DETACH) {
        g_orderWord = (g_orderWord << 8) | g_nextOrder++;
    }
}

BOOL WINAPI DllMain(HANDLE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        ++g_attachCount;
        g_orderWord = (g_orderWord << 8) | g_nextOrder++;
    } else if (reason == DLL_PROCESS_DETACH) {
        g_orderWord = (g_orderWord << 8) | g_nextOrder++;
        if (g_detachOrderObserved) {
            *g_detachOrderObserved = g_orderWord;
        }
    }
    return TRUE;
}

extern "C" DWORD WINAPI TlsCallbackCount() {
    return g_tlsCount;
}

extern "C" DWORD WINAPI TlsAttachCount() {
    return g_attachCount;
}

extern "C" DWORD WINAPI TlsOrderWord() {
    return g_orderWord;
}

extern "C" void WINAPI TlsArmDetachOrderPointer(volatile DWORD *value) {
    g_detachOrderObserved = value;
}

extern "C" DWORD _tls_index = 0;

#pragma data_seg(".tls")
extern "C" DWORD _tls_start = 0;
#pragma data_seg(".tls$ZZZ")
extern "C" DWORD _tls_end = 0;
#pragma data_seg()

#pragma const_seg(".CRT$XLB")
extern "C" const PFN_TLS_CALLBACK FixtureTlsCallbacks[] = {
    FixtureTlsCallback,
    0,
};
#pragma const_seg(".rdata$T")
extern "C" const IMAGE_TLS_DIRECTORY32_FIXTURE _tls_used = {
    (DWORD)&_tls_start,
    (DWORD)&_tls_end,
    (DWORD)&_tls_index,
    (DWORD)FixtureTlsCallbacks,
    0,
    0,
};
#pragma const_seg()

#pragma comment(linker, "/include:_tls_used")
