#include <windows.h>
#include "../common/fixture_status.h"

#ifndef FILE_MAP_ALL_ACCESS
#define FILE_MAP_ALL_ACCESS 0x000f001f
#endif

static const wchar_t* MAP_NAME = L"Fixture036RelativeProcessMapW";
static const wchar_t* EVT_NAME = L"Fixture036RelativeProcessEventW";

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE mapping = CreateFileMappingW(INVALID_HANDLE_VALUE, 0, PAGE_READWRITE, 0, 4096, MAP_NAME);
    if (!mapping) return FixtureFail(3701);

    BYTE* view = (BYTE*)MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096);
    if (!view) return FixtureFail(3702);

    if (view[0] != 'P' || view[1] != 'A' || view[2] != 'R' || view[3] != 'E') {
        return FixtureFail(3703);
    }

    view[16] = 'C';
    view[17] = 'H';
    view[18] = 'I';
    view[19] = 'L';
    view[20] = 'D';
    FlushViewOfFile(view, 32);

    HANDLE eventHandle = CreateEventW(0, TRUE, FALSE, EVT_NAME);
    if (!eventHandle) return FixtureFail(3704);

    if (!SetEvent(eventHandle)) return FixtureFail(3705);

    CloseHandle(eventHandle);
    UnmapViewOfFile(view);
    CloseHandle(mapping);
    return FIXTURE_OK;
}
