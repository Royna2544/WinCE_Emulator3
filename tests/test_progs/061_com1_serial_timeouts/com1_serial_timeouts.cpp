#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE com = CreateFileW(L"COM1:", GENERIC_READ | GENERIC_WRITE, 0, 0, OPEN_EXISTING, 0, 0);
    if (com == INVALID_HANDLE_VALUE) return FixtureFail(6101);

    DCB dcb;
    ZeroMemory(&dcb, sizeof(dcb));
    dcb.DCBlength = sizeof(dcb);

    if (!GetCommState(com, &dcb)) return FixtureFail(6102);

    dcb.BaudRate = 9600;
    dcb.ByteSize = 8;
    dcb.Parity = NOPARITY;
    dcb.StopBits = ONESTOPBIT;

    if (!SetCommState(com, &dcb)) return FixtureFail(6103);

    COMMTIMEOUTS timeouts;
    ZeroMemory(&timeouts, sizeof(timeouts));
    timeouts.ReadIntervalTimeout = 50;
    timeouts.ReadTotalTimeoutConstant = 10;
    timeouts.WriteTotalTimeoutConstant = 10;

    if (!SetCommTimeouts(com, &timeouts)) return FixtureFail(6104);
    if (!SetupComm(com, 256, 256)) return FixtureFail(6105);

    DWORD errors = 0;
    COMSTAT stat;
    ZeroMemory(&stat, sizeof(stat));
    ClearCommError(com, &errors, &stat);

    PurgeComm(com, PURGE_RXCLEAR | PURGE_TXCLEAR);
    CloseHandle(com);
    return FIXTURE_OK;
}
