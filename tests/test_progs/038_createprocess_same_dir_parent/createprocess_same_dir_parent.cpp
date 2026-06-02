#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* CHILD_REL = L"039_exit_marker_child.exe";
static const wchar_t* MARKER = L"\\SDMMC Disk\\fixture_039_marker.tmp";

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    DeleteFileW(MARKER);

    PROCESS_INFORMATION pi;
    STARTUPINFO si;
    ZeroMemory(&pi, sizeof(pi));
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);

    BOOL ok = CreateProcessW(CHILD_REL, 0, 0, 0, FALSE, 0, 0, 0, &si, &pi);
    if (!ok) return FixtureFail(3801);

    WaitForSingleObject(pi.hProcess, 5000);

    DWORD spins = 0;
    while (!FileExistsW(MARKER) && spins < 100) {
        Sleep(10);
        ++spins;
    }

    if (!FileExistsW(MARKER)) return FixtureFail(3802);

    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    DeleteFileW(MARKER);
    return FIXTURE_OK;
}
