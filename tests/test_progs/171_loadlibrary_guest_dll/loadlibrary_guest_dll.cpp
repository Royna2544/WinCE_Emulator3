#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFN_FIXTURE_EXPORT)(DWORD);

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HMODULE module = LoadLibraryW(L"runtime_fixture.dll");
    if (!module) return FixtureFail(17101);

    HMODULE retained = LoadLibraryW(L"runtime_fixture.dll");
    if (!retained) return FixtureFail(17102);
    if (retained != module) return FixtureFail(17103);

    FARPROC namedProc = GetProcAddressW(module, L"FixtureNamedExport");
    if (!namedProc) return FixtureFail(17104);
    PFN_FIXTURE_EXPORT named = (PFN_FIXTURE_EXPORT)namedProc;
    if (named(0x1200) != 0x4634) return FixtureFail(17105);

    FARPROC ordinalProc = GetProcAddressW(module, (LPCWSTR)7);
    if (!ordinalProc) return FixtureFail(17106);
    PFN_FIXTURE_EXPORT ordinal = (PFN_FIXTURE_EXPORT)ordinalProc;
    if (ordinal(0x30) != 0x7077) return FixtureFail(17107);

    if (!FreeLibrary(retained)) return FixtureFail(17108);
    if (!FreeLibrary(module)) return FixtureFail(17109);

    return FIXTURE_OK;
}
