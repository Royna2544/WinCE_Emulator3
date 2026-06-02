#include <windows.h>
#include "../common/fixture_status.h"

#ifndef FILE_MAP_ALL_ACCESS
#define FILE_MAP_ALL_ACCESS 0x000f001f
#endif

static const wchar_t* MAP_NAME = L"Fixture040CommandLineMapW";
static const wchar_t* EVT_NAME = L"Fixture040CommandLineEventW";
static const wchar_t* CHILD_REL = L"..\\041_commandline_child\\041_commandline_child.exe";
static wchar_t CMD_LINE[] = L"..\\041_commandline_child\\041_commandline_child.exe alpha beta 123";

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE mapping = CreateFileMappingW(INVALID_HANDLE_VALUE, 0, PAGE_READWRITE, 0, 4096, MAP_NAME);
    if (!mapping) return FixtureFail(4001);

    BYTE* view = (BYTE*)MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
    if (!view) return FixtureFail(4002);

    HANDLE eventHandle = CreateEventW(0, TRUE, FALSE, EVT_NAME);
    if (!eventHandle) return FixtureFail(4003);

    PROCESS_INFORMATION pi;
    STARTUPINFO si;
    ZeroMemory(&pi, sizeof(pi));
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);

    BOOL ok = CreateProcessW(CHILD_REL, CMD_LINE, 0, 0, FALSE, 0, 0, 0, &si, &pi);
    if (!ok) return FixtureFail(4004);

    if (WaitForSingleObject(eventHandle, 5000) != WAIT_OBJECT_0) return FixtureFail(4005);

    if (view[0] != 'O' || view[1] != 'K') return FixtureFail(4006);

    WaitForSingleObject(pi.hProcess, 5000);
    DWORD exitCode = 0xffffffff;
    if (!GetExitCodeProcess(pi.hProcess, &exitCode)) return FixtureFail(4007);
    if (exitCode != 0) return FixtureFail(4008);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    CloseHandle(eventHandle);
    UnmapViewOfFile(view);
    CloseHandle(mapping);
    return FIXTURE_OK;
}
