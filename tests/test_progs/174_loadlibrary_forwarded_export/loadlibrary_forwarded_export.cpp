#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFN_FORWARD)(DWORD);

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HMODULE forwarder = LoadLibraryW(L"b_forwarder.dll");
    if (!forwarder) return FixtureFail(17401);

    FARPROC byNameProc = GetProcAddressW(forwarder, L"ForwardByName");
    if (!byNameProc) return FixtureFail(17402);
    PFN_FORWARD byName = (PFN_FORWARD)byNameProc;
    if (byName(5) != 0x1205) return FixtureFail(17403);

    FARPROC byOrdinalProc = GetProcAddressW(forwarder, (LPCWSTR)4);
    if (!byOrdinalProc) return FixtureFail(17404);
    PFN_FORWARD byOrdinal = (PFN_FORWARD)byOrdinalProc;
    if (byOrdinal(6) != 0x1206) return FixtureFail(17405);

    HMODULE user = LoadLibraryW(L"c_forward_user.dll");
    if (!user) return FixtureFail(17406);
    FARPROC userProc = GetProcAddressW(user, L"UserCallsForward");
    if (!userProc) return FixtureFail(17407);
    PFN_FORWARD userCallsForward = (PFN_FORWARD)userProc;
    if (userCallsForward(7) != 0x125c) return FixtureFail(17408);

    if (!FreeLibrary(user)) return FixtureFail(17409);
    if (!FreeLibrary(forwarder)) return FixtureFail(17410);
    return FIXTURE_OK;
}
