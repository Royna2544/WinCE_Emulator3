#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HKEY root = 0;
    DWORD disp = 0;
    LONG rc = RegCreateKeyExW(HKEY_CURRENT_USER, L"Software\\FixtureRegEnum", 0, 0, 0, 0, 0, &root, &disp);
    if (rc != ERROR_SUCCESS) return FixtureFail(9701);

    HKEY child = 0;
    rc = RegCreateKeyExW(root, L"ChildA", 0, 0, 0, 0, 0, &child, &disp);
    if (rc != ERROR_SUCCESS) return FixtureFail(9702);
    RegCloseKey(child);

    const wchar_t expand[] = L"%SDMMC%\\file";
    rc = RegSetValueExW(root, L"Expand", 0, REG_EXPAND_SZ, (const BYTE*)expand, sizeof(expand));
    if (rc != ERROR_SUCCESS) return FixtureFail(9703);

    wchar_t name[64];
    DWORD nameLen = 64;
    rc = RegEnumKeyExW(root, 0, name, &nameLen, 0, 0, 0, 0);
    if (rc != ERROR_SUCCESS) return FixtureFail(9704);

    DWORD subkeys = 0;
    DWORD values = 0;
    rc = RegQueryInfoKeyW(root, 0, 0, 0, &subkeys, 0, 0, &values, 0, 0, 0, 0);
    if (rc != ERROR_SUCCESS || subkeys < 1 || values < 1) return FixtureFail(9705);

    RegDeleteKeyW(root, L"ChildA");
    RegDeleteValueW(root, L"Expand");
    RegCloseKey(root);
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\FixtureRegEnum");
    return FIXTURE_OK;
}
