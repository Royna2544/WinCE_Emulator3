#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFN_COUNT)();

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HMODULE module = LoadLibraryW(L"tls_fixture.dll");
    if (!module) return FixtureFail(17301);

    FARPROC tlsProc = GetProcAddressW(module, L"TlsCallbackCount");
    FARPROC attachProc = GetProcAddressW(module, L"TlsAttachCount");
    FARPROC orderProc = GetProcAddressW(module, L"TlsOrderWord");
    if (!tlsProc || !attachProc || !orderProc) return FixtureFail(17302);

    PFN_COUNT tlsCount = (PFN_COUNT)tlsProc;
    PFN_COUNT attachCount = (PFN_COUNT)attachProc;
    PFN_COUNT orderWord = (PFN_COUNT)orderProc;

    if (tlsCount() != 1) return FixtureFail(17303);
    if (attachCount() != 1) return FixtureFail(17304);
    if (orderWord() != 0x0102) return FixtureFail(17305);

    if (!FreeLibrary(module)) return FixtureFail(17306);
    return FIXTURE_OK;
}
