#include <windows.h>
#include "../common/fixture_status.h"

static const wchar_t* MARKER = L"\\SDMMC Disk\\fixture_039_marker.tmp";

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    if (!WriteMarkerFileW(MARKER, "child-ran")) return FixtureFail(3901);
    return FIXTURE_OK;
}
