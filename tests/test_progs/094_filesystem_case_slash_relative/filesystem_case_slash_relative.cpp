#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    CreateDirectoryW(L"\\SDMMC Disk\\FixtureCase", 0);

    HANDLE f = CreateFileW(L"\\SDMMC Disk\\FixtureCase\\MixedName.TXT", GENERIC_WRITE | GENERIC_READ, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(9401);
    CloseHandle(f);

    if (!FileExistsW(L"\\SDMMC Disk\\fixturecase\\mixedname.txt")) return FixtureFail(9402);
    if (!FileExistsW(L"\\SDMMC Disk/FixtureCase/MixedName.TXT")) return FixtureFail(9403);

    DeleteFileW(L"\\SDMMC Disk\\FixtureCase\\MixedName.TXT");
    return FIXTURE_OK;
}
