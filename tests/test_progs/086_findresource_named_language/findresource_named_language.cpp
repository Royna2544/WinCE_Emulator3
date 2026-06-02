#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    HRSRC res = FindResourceW(h, L"CUSTOM_DATA", MAKEINTRESOURCEW(10));
    if (!res) return FixtureFail(8601);

    DWORD size = SizeofResource(h, res);
    if (size < 5) return FixtureFail(8602);

    HGLOBAL data = LoadResource(h, res);
    if (!data) return FixtureFail(8603);

    wchar_t str[64];
    ZeroMemory(str, sizeof(str));
    if (LoadStringW(h, IDS_NAMED_RESOURCE, str, 64) <= 0) return FixtureFail(8604);
    if (!WideEqAscii(str, "named-resource-ok")) return FixtureFail(8605);

    return FIXTURE_OK;
}
