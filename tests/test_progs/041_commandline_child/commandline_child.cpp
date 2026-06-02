#include <windows.h>
#include "../common/fixture_status.h"

#ifndef FILE_MAP_ALL_ACCESS
#define FILE_MAP_ALL_ACCESS 0x000f001f
#endif

static const wchar_t* MAP_NAME = L"Fixture040CommandLineMapW";
static const wchar_t* EVT_NAME = L"Fixture040CommandLineEventW";

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR lpCmdLine, int) {
    if (!WideContains(lpCmdLine, L"alpha")) return FixtureFail(4101);
    if (!WideContains(lpCmdLine, L"beta")) return FixtureFail(4102);
    if (!WideContains(lpCmdLine, L"123")) return FixtureFail(4103);

    HANDLE mapping = CreateFileMappingW(INVALID_HANDLE_VALUE, 0, PAGE_READWRITE, 0, 4096, MAP_NAME);
    if (!mapping) return FixtureFail(4104);

    BYTE* view = (BYTE*)MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
    if (!view) return FixtureFail(4105);

    view[0] = 'O';
    view[1] = 'K';

    HANDLE eventHandle = CreateEventW(0, TRUE, FALSE, EVT_NAME);
    if (!eventHandle) return FixtureFail(4106);

    SetEvent(eventHandle);
    CloseHandle(eventHandle);
    UnmapViewOfFile(view);
    CloseHandle(mapping);
    return FIXTURE_OK;
}
