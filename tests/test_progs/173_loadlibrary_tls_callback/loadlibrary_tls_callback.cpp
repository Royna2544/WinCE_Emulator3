#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFN_COUNT)();
typedef void (WINAPI *PFN_ARM_DETACH_ORDER)(volatile DWORD *);

static volatile DWORD g_detachOrderObserved = 0;

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HMODULE module = LoadLibraryW(L"tls_fixture.dll");
    if (!module) return FixtureFail(17301);

    FARPROC tlsProc = GetProcAddressW(module, L"TlsCallbackCount");
    FARPROC attachProc = GetProcAddressW(module, L"TlsAttachCount");
    FARPROC orderProc = GetProcAddressW(module, L"TlsOrderWord");
    FARPROC armProc = GetProcAddressW(module, L"TlsArmDetachOrderPointer");
    if (!tlsProc || !attachProc || !orderProc || !armProc) return FixtureFail(17302);

    PFN_COUNT tlsCount = (PFN_COUNT)tlsProc;
    PFN_COUNT attachCount = (PFN_COUNT)attachProc;
    PFN_COUNT orderWord = (PFN_COUNT)orderProc;
    PFN_ARM_DETACH_ORDER armDetachOrder = (PFN_ARM_DETACH_ORDER)armProc;

    if (tlsCount() != 1) return FixtureFail(17303);
    if (attachCount() != 1) return FixtureFail(17304);
    if (orderWord() != 0x0102) return FixtureFail(17305);
    armDetachOrder(&g_detachOrderObserved);

    if (!FreeLibrary(module)) return FixtureFail(17306);
    if (g_detachOrderObserved != 0x01020304) return FixtureFail(17307);
    return FIXTURE_OK;
}
