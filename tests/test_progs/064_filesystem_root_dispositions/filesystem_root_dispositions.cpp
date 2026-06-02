#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    const wchar_t* dir = L"\\SDMMC Disk\\fixture_dispositions";
    const wchar_t* path = L"\\SDMMC Disk\\fixture_dispositions\\disp.txt";
    DeleteFileW(path);
    CreateDirectoryW(dir, 0);

    HANDLE f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(6401);
    CloseHandle(f);

    f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, 0);
    if (f != INVALID_HANDLE_VALUE) return FixtureFail(6402);

    f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(6403);
    CloseHandle(f);

    f = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, 0, 0, TRUNCATE_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(6404);
    if (GetFileSize(f, 0) != 0) return FixtureFail(6405);
    CloseHandle(f);

    WIN32_FILE_ATTRIBUTE_DATA attrs;
    if (!GetFileAttributesExW(path, GetFileExInfoStandard, &attrs)) return FixtureFail(6406);

    WIN32_FIND_DATAW data;
    HANDLE find = FindFirstFileW(L"\\SDMMC Disk\\*", &data);
    if (find == INVALID_HANDLE_VALUE) return FixtureFail(6407);
    FindClose(find);

    DeleteFileW(path);
    return FIXTURE_OK;
}
