#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    const wchar_t* path = L"\\SDMMC Disk\\fixture_file_io.tmp";
    const char payload[] = "fixture-io";
    char readBack[32];

    HANDLE file = CreateFileW(
        path,
        GENERIC_READ | GENERIC_WRITE,
        0,
        0,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        0
    );

    if (file == INVALID_HANDLE_VALUE) {
        return FixtureFail(1);
    }

    DWORD written = 0;
    if (!WriteFile(file, payload, sizeof(payload), &written, 0)) {
        CloseHandle(file);
        return FixtureFail(2);
    }
    if (written != sizeof(payload)) {
        CloseHandle(file);
        return FixtureFail(3);
    }

    if (SetFilePointer(file, 0, 0, FILE_BEGIN) == 0xffffffff) {
        CloseHandle(file);
        return FixtureFail(4);
    }

    ZeroMemory(readBack, sizeof(readBack));
    DWORD read = 0;
    if (!ReadFile(file, readBack, sizeof(payload), &read, 0)) {
        CloseHandle(file);
        return FixtureFail(5);
    }
    if (read != sizeof(payload)) {
        CloseHandle(file);
        return FixtureFail(6);
    }

    for (DWORD i = 0; i < sizeof(payload); ++i) {
        if (readBack[i] != payload[i]) {
            CloseHandle(file);
            return FixtureFail(7);
        }
    }

    CloseHandle(file);
    DeleteFileW(path);

    return FIXTURE_OK;
}
