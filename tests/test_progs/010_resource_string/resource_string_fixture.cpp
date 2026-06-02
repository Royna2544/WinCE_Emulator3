#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

static int WideEqualsAscii(const wchar_t* wide, const char* ascii) {
    int i = 0;
    while (wide[i] && ascii[i]) {
        if (wide[i] != (wchar_t)(unsigned char)ascii[i]) {
            return 0;
        }
        ++i;
    }
    return wide[i] == 0 && ascii[i] == 0;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int) {
    wchar_t buffer[128];
    ZeroMemory(buffer, sizeof(buffer));

    int len = LoadStringW(hInstance, IDS_FIXTURE_HELLO, buffer, 128);
    if (len <= 0) {
        return FixtureFail(1);
    }

    if (!WideEqualsAscii(buffer, "fixture-resource-ok")) {
        return FixtureFail(2);
    }

    return FIXTURE_OK;
}
