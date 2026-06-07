#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFN_DEPENDENT_EXPORT)(DWORD);
typedef DWORD (WINAPI *PFN_DEPENDENT_COUNT)();

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HMODULE user = LoadLibraryW(L"dependency_user.dll");
    if (!user) return FixtureFail(17201);

    FARPROC proc = GetProcAddressW(user, L"DependentUserExport");
    if (!proc) return FixtureFail(17202);

    PFN_DEPENDENT_EXPORT call = (PFN_DEPENDENT_EXPORT)proc;
    if (call(0x22) != 0x5357) return FixtureFail(17203);

    FARPROC userAttachProc = GetProcAddressW(user, L"DependencyUserAttachCount");
    if (!userAttachProc) return FixtureFail(17205);
    PFN_DEPENDENT_COUNT userAttach = (PFN_DEPENDENT_COUNT)userAttachProc;
    if (userAttach() != 1) return FixtureFail(17206);

    HMODULE base = LoadLibraryW(L"dependency_base.dll");
    if (!base) return FixtureFail(17207);
    FARPROC baseAttachProc = GetProcAddressW(base, L"DependencyBaseAttachCount");
    if (!baseAttachProc) return FixtureFail(17208);
    PFN_DEPENDENT_COUNT baseAttach = (PFN_DEPENDENT_COUNT)baseAttachProc;
    if (baseAttach() != 1) return FixtureFail(17209);

    if (!FreeLibrary(base)) return FixtureFail(17210);
    if (!FreeLibrary(user)) return FixtureFail(17204);
    return FIXTURE_OK;
}
