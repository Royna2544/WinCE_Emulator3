#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE com = CreateFileW(L"COM1:", GENERIC_READ | GENERIC_WRITE, 0, 0, OPEN_EXISTING, 0, 0);
    if (com == INVALID_HANDLE_VALUE) return FixtureFail(6201);

    BYTE inbuf[4] = {1,2,3,4};
    BYTE outbuf[8];
    DWORD returned = 0;

    BOOL ok = DeviceIoControl(com, 0x12345678, inbuf, sizeof(inbuf), outbuf, sizeof(outbuf), &returned, 0);

    /*
       Unknown IOCTL may fail. The important contract is that it returns
       deterministically and does not corrupt process state.
    */
    (void)ok;
    (void)returned;

    CloseHandle(com);
    return FIXTURE_OK;
}
