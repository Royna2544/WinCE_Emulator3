#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HKEY key = 0;
    DWORD disp = 0;
    LONG rc = RegCreateKeyExW(HKEY_CURRENT_USER, L"Software\\FixtureRegistryEdge", 0, 0, 0, 0, 0, &key, &disp);
    if (rc != ERROR_SUCCESS) return FixtureFail(6301);

    BYTE bin[] = {0xde,0xad,0xbe,0xef};
    rc = RegSetValueExW(key, L"Binary", 0, REG_BINARY, bin, sizeof(bin));
    if (rc != ERROR_SUCCESS) return FixtureFail(6302);

    const wchar_t multi[] = L"one\0two\0three\0\0";
    rc = RegSetValueExW(key, L"Multi", 0, REG_MULTI_SZ, (const BYTE*)multi, sizeof(multi));
    if (rc != ERROR_SUCCESS) return FixtureFail(6303);

    DWORD subkeys = 0, values = 0;
    rc = RegQueryInfoKeyW(key, 0, 0, 0, &subkeys, 0, 0, &values, 0, 0, 0, 0);
    if (rc != ERROR_SUCCESS) return FixtureFail(6304);
    if (values < 2) return FixtureFail(6305);

    wchar_t name[64];
    DWORD nameLen = 64;
    DWORD type = 0;
    BYTE data[128];
    DWORD dataLen = sizeof(data);

    rc = RegEnumValueW(key, 0, name, &nameLen, 0, &type, data, &dataLen);
    if (rc != ERROR_SUCCESS) return FixtureFail(6306);

    dataLen = sizeof(data);
    type = 0;
    rc = RegQueryValueExW(key, L"Binary", 0, &type, data, &dataLen);
    if (rc != ERROR_SUCCESS || type != REG_BINARY || dataLen != sizeof(bin) || !BytesEq(data, bin, sizeof(bin))) {
        return FixtureFail(6307);
    }

    RegDeleteValueW(key, L"Binary");
    RegDeleteValueW(key, L"Multi");
    RegCloseKey(key);
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\FixtureRegistryEdge");
    return FIXTURE_OK;
}
