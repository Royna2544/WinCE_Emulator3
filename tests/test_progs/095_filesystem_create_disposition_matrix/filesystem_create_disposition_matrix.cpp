#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    const wchar_t* path = L"\\SDMMC Disk\\fixture_disposition_matrix.tmp";
    DeleteFileW(path);

    HANDLE f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
    if (f != INVALID_HANDLE_VALUE) return FixtureFail(9501);

    f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(9502);
    CloseHandle(f);

    f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(9503);
    CloseHandle(f);

    f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, TRUNCATE_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(9504);
    CloseHandle(f);

    DeleteFileW(path);
    return FIXTURE_OK;
}
