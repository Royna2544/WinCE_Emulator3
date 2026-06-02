#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFNGETTICKCOUNT)();

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int) {
    HMODULE self = GetModuleHandleW(0);
    if (!self) return FixtureFail(6501);

    wchar_t path[260];
    ZeroMemory(path, sizeof(path));
    DWORD len = GetModuleFileNameW(self, path, 260);
    if (len == 0) return FixtureFail(6502);

    HMODULE core = LoadLibraryW(L"coredll.dll");
    if (!core) return FixtureFail(6503);

    FARPROC procW = GetProcAddressW(core, L"GetTickCount");
    if (!procW) return FixtureFail(6504);

    PFNGETTICKCOUNT getTick = (PFNGETTICKCOUNT)procW;
    DWORD t0 = getTick();
    Sleep(1);
    DWORD t1 = getTick();
    if (t1 < t0) return FixtureFail(6505);

    FARPROC procA = GetProcAddressA(core, "GetTickCount");
    if (!procA) return FixtureFail(6506);

    return FIXTURE_OK;
}
