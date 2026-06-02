#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HKEY key = 0;
    DWORD disp = 0;
    LONG rc = RegCreateKeyExW(HKEY_CURRENT_USER, L"Software\\FixtureCaseReg", 0, 0, 0, 0, 0, &key, &disp);
    if (rc != ERROR_SUCCESS) return FixtureFail(9801);

    DWORD value = 0x12345678;
    rc = RegSetValueExW(key, L"MixedValue", 0, REG_DWORD, (const BYTE*)&value, sizeof(value));
    if (rc != ERROR_SUCCESS) return FixtureFail(9802);
    RegCloseKey(key);

    HKEY opened = 0;
    rc = RegOpenKeyExW(HKEY_CURRENT_USER, L"software\\fixturecasereg", 0, 0, &opened);
    if (rc != ERROR_SUCCESS) return FixtureFail(9803);

    DWORD type = 0;
    DWORD out = 0;
    DWORD cb = sizeof(out);
    rc = RegQueryValueExW(opened, L"mixedvalue", 0, &type, (BYTE*)&out, &cb);
    if (rc != ERROR_SUCCESS || type != REG_DWORD || out != value) return FixtureFail(9804);

    cb = sizeof(out);
    rc = RegQueryValueExW(opened, L"MissingValue", 0, &type, (BYTE*)&out, &cb);
    if (rc == ERROR_SUCCESS) return FixtureFail(9805);

    RegDeleteValueW(opened, L"MixedValue");
    RegCloseKey(opened);
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\FixtureCaseReg");
    return FIXTURE_OK;
}
