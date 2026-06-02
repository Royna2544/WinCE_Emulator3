#include <windows.h>
#include "../common/fixture_status.h"

#ifndef FILE_MAP_ALL_ACCESS
#define FILE_MAP_ALL_ACCESS 0x000f001f
#endif

static const wchar_t* MAP_NAME = L"Fixture036RelativeProcessMapW";
static const wchar_t* EVT_NAME = L"Fixture036RelativeProcessEventW";
static const wchar_t* CHILD_REL = L"..\\037_ipc_child\\037_ipc_child.exe";

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE mapping = CreateFileMappingW(INVALID_HANDLE_VALUE, 0, PAGE_READWRITE, 0, 4096, MAP_NAME);
    if (!mapping) return FixtureFail(3601);

    BYTE* view = (BYTE*)MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
    if (!view) return FixtureFail(3602);

    view[0] = 'P';
    view[1] = 'A';
    view[2] = 'R';
    view[3] = 'E';
    view[4] = 'N';
    view[5] = 'T';

    HANDLE eventHandle = CreateEventW(0, TRUE, FALSE, EVT_NAME);
    if (!eventHandle) return FixtureFail(3603);

    PROCESS_INFORMATION pi;
    STARTUPINFO si;
    ZeroMemory(&pi, sizeof(pi));
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);

    /*
       Intentional relative launch path:
       ..\037_ipc_child\037_ipc_child.exe
    */
    BOOL ok = CreateProcessW(CHILD_REL, 0, 0, 0, FALSE, 0, 0, 0, &si, &pi);
    if (!ok) return FixtureFail(3604);

    if (WaitForSingleObject(eventHandle, 5000) != WAIT_OBJECT_0) return FixtureFail(3605);

    if (view[16] != 'C' || view[17] != 'H' || view[18] != 'I' || view[19] != 'L' || view[20] != 'D') {
        return FixtureFail(3606);
    }

    WaitForSingleObject(pi.hProcess, 5000);

    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    CloseHandle(eventHandle);
    UnmapViewOfFile(view);
    CloseHandle(mapping);
    return FIXTURE_OK;
}
