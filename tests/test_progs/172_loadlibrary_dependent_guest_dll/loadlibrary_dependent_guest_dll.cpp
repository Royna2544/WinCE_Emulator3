#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFN_DEPENDENT_EXPORT)(DWORD);

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HMODULE user = LoadLibraryW(L"dependency_user.dll");
    if (!user) return FixtureFail(17201);

    FARPROC proc = GetProcAddressW(user, L"DependentUserExport");
    if (!proc) return FixtureFail(17202);

    PFN_DEPENDENT_EXPORT call = (PFN_DEPENDENT_EXPORT)proc;
    if (call(0x22) != 0x5357) return FixtureFail(17203);

    if (!FreeLibrary(user)) return FixtureFail(17204);
    return FIXTURE_OK;
}
