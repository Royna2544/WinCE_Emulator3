#include <windows.h>
#include "../common/fixture_status.h"

#ifndef DONT_RESOLVE_DLL_REFERENCES
#define DONT_RESOLVE_DLL_REFERENCES 0x00000001
#endif

typedef DWORD (WINAPI *PFN_COUNT)();
typedef DWORD (WINAPI *PFN_VALUE)();

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    if (GetModuleHandleW(L"a_noresolve_dependency.dll")) return FixtureFail(17501);

    HMODULE noResolve = LoadLibraryExW(
        L"b_noresolve_user.dll",
        NULL,
        DONT_RESOLVE_DLL_REFERENCES);
    if (!noResolve) return FixtureFail(17502);
    if (GetModuleHandleW(L"a_noresolve_dependency.dll")) return FixtureFail(17503);

    PFN_COUNT attachCount = (PFN_COUNT)GetProcAddressW(noResolve, L"NoResolveAttachCount");
    if (!attachCount) return FixtureFail(17504);
    if (attachCount() != 0) return FixtureFail(17505);

    PFN_COUNT detachCount = (PFN_COUNT)GetProcAddressW(noResolve, L"NoResolveDetachCount");
    if (!detachCount) return FixtureFail(17506);
    if (detachCount() != 0) return FixtureFail(17507);

    PFN_VALUE plainValue = (PFN_VALUE)GetProcAddressW(noResolve, L"NoResolvePlainValue");
    if (!plainValue) return FixtureFail(17508);
    if (plainValue() != 0x4455) return FixtureFail(17509);

    if (!FreeLibrary(noResolve)) return FixtureFail(17510);
    if (GetModuleHandleW(L"a_noresolve_dependency.dll")) return FixtureFail(17511);

    HMODULE normal = LoadLibraryW(L"b_noresolve_user.dll");
    if (!normal) return FixtureFail(17512);
    if (!GetModuleHandleW(L"a_noresolve_dependency.dll")) return FixtureFail(17513);

    attachCount = (PFN_COUNT)GetProcAddressW(normal, L"NoResolveAttachCount");
    if (!attachCount) return FixtureFail(17514);
    if (attachCount() != 1) return FixtureFail(17515);

    PFN_VALUE importedValue = (PFN_VALUE)GetProcAddressW(normal, L"NoResolveImportedValue");
    if (!importedValue) return FixtureFail(17516);
    if (importedValue() != 0x7788) return FixtureFail(17517);

    if (!FreeLibrary(normal)) return FixtureFail(17518);
    return FIXTURE_OK;
}
