#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

#ifndef LOAD_LIBRARY_AS_DATAFILE
#define LOAD_LIBRARY_AS_DATAFILE 0x00000002
#endif

typedef DWORD (WINAPI *PFN_COUNT)();

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    if (GetModuleHandleW(L"a_datafile_dependency.dll")) return FixtureFail(17601);

    HMODULE datafile = LoadLibraryExW(
        L"b_datafile_resource.dll",
        NULL,
        LOAD_LIBRARY_AS_DATAFILE);
    if (!datafile) return FixtureFail(17602);
    if (GetModuleHandleW(L"a_datafile_dependency.dll")) return FixtureFail(17603);
    if (GetProcAddressW(datafile, L"DatafileAttachCount")) return FixtureFail(17604);

    wchar_t text[64];
    ZeroMemory(text, sizeof(text));
    if (LoadStringW(datafile, IDS_DATAFILE_STRING, text, 64) <= 0) return FixtureFail(17605);
    if (!WideEqAscii(text, "datafile-resource-ok")) return FixtureFail(17606);

    HRSRC res = FindResourceW(datafile, L"DATAFILE_BLOB", MAKEINTRESOURCEW(10));
    if (!res) return FixtureFail(17607);
    if (SizeofResource(datafile, res) < 10) return FixtureFail(17608);
    BYTE *bytes = (BYTE *)LoadResource(datafile, res);
    if (!bytes) return FixtureFail(17609);
    if (bytes[0] != 0x44 || bytes[2] != 0x41 || bytes[4] != 0x54 ||
        bytes[6] != 0x41 || bytes[8] != 0x21) {
        return FixtureFail(17610);
    }

    if (!FreeLibrary(datafile)) return FixtureFail(17611);
    if (GetModuleHandleW(L"a_datafile_dependency.dll")) return FixtureFail(17612);

    HMODULE normal = LoadLibraryW(L"b_datafile_resource.dll");
    if (!normal) return FixtureFail(17613);
    if (!GetModuleHandleW(L"a_datafile_dependency.dll")) return FixtureFail(17614);

    PFN_COUNT attachCount = (PFN_COUNT)GetProcAddressW(normal, L"DatafileAttachCount");
    if (!attachCount) return FixtureFail(17615);
    if (attachCount() != 1) return FixtureFail(17616);

    if (!FreeLibrary(normal)) return FixtureFail(17617);
    return FIXTURE_OK;
}
