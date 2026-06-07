#include <windows.h>
#include "../common/fixture_status.h"

typedef DWORD (WINAPI *PFN_FIXTURE_EXPORT)(DWORD);
typedef DWORD (WINAPI *PFN_FIXTURE_COUNT)();
typedef void (WINAPI *PFN_FIXTURE_ARM_DETACH)(volatile DWORD *);

static volatile DWORD g_detachObserved = 0;

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HMODULE module = LoadLibraryW(L"runtime_fixture.dll");
    if (!module) return FixtureFail(17101);

    FARPROC attachProc = GetProcAddressW(module, L"FixtureAttachCount");
    if (!attachProc) return FixtureFail(17110);
    PFN_FIXTURE_COUNT attachCount = (PFN_FIXTURE_COUNT)attachProc;
    if (attachCount() != 1) return FixtureFail(17111);
    FARPROC detachProc = GetProcAddressW(module, L"FixtureDetachCount");
    if (!detachProc) return FixtureFail(17113);
    PFN_FIXTURE_COUNT detachCount = (PFN_FIXTURE_COUNT)detachProc;
    if (detachCount() != 0) return FixtureFail(17114);
    FARPROC armDetachProc = GetProcAddressW(module, L"FixtureArmDetachPointer");
    if (!armDetachProc) return FixtureFail(17115);
    PFN_FIXTURE_ARM_DETACH armDetach = (PFN_FIXTURE_ARM_DETACH)armDetachProc;
    armDetach(&g_detachObserved);

    HMODULE retained = LoadLibraryW(L"runtime_fixture.dll");
    if (!retained) return FixtureFail(17102);
    if (retained != module) return FixtureFail(17103);
    if (attachCount() != 1) return FixtureFail(17112);

    FARPROC namedProc = GetProcAddressW(module, L"FixtureNamedExport");
    if (!namedProc) return FixtureFail(17104);
    PFN_FIXTURE_EXPORT named = (PFN_FIXTURE_EXPORT)namedProc;
    if (named(0x1200) != 0x4634) return FixtureFail(17105);

    FARPROC ordinalProc = GetProcAddressW(module, (LPCWSTR)7);
    if (!ordinalProc) return FixtureFail(17106);
    PFN_FIXTURE_EXPORT ordinal = (PFN_FIXTURE_EXPORT)ordinalProc;
    if (ordinal(0x30) != 0x7077) return FixtureFail(17107);

    if (!FreeLibrary(retained)) return FixtureFail(17108);
    if (detachCount() != 0) return FixtureFail(17116);
    if (g_detachObserved != 0) return FixtureFail(17117);
    if (!FreeLibrary(module)) return FixtureFail(17109);
    if (g_detachObserved != 1) return FixtureFail(17118);

    return FIXTURE_OK;
}
