#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    const wchar_t* dir = L"\\SDMMC Disk\\fixture_attr_dir";
    const wchar_t* file = L"\\SDMMC Disk\\fixture_attr_dir\\time.txt";

    CreateDirectoryW(dir, 0);

    HANDLE f = CreateFileW(file, GENERIC_READ | GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (f == INVALID_HANDLE_VALUE) return FixtureFail(9601);

    SYSTEMTIME st;
    FILETIME ft;
    ZeroMemory(&st, sizeof(st));
    st.wYear = 2020;
    st.wMonth = 1;
    st.wDay = 2;
    st.wHour = 3;
    st.wMinute = 4;
    st.wSecond = 5;

    if (!SystemTimeToFileTime(&st, &ft)) return FixtureFail(9602);
    SetFileTime(f, &ft, &ft, &ft);
    CloseHandle(f);

    WIN32_FILE_ATTRIBUTE_DATA data;
    if (!GetFileAttributesExW(file, GetFileExInfoStandard, &data)) return FixtureFail(9603);

    DWORD attrs = GetFileAttributesW(dir);
    if (attrs == 0xffffffff || (attrs & FILE_ATTRIBUTE_DIRECTORY) == 0) return FixtureFail(9604);

    DeleteFileW(file);
    return FIXTURE_OK;
}
