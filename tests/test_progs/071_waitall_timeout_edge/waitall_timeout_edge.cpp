#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE e1 = CreateEventW(0, TRUE, TRUE, 0);
    HANDLE e2 = CreateEventW(0, TRUE, FALSE, 0);
    if (!e1 || !e2) return FixtureFail(7101);

    HANDLE handles[2] = { e1, e2 };

    // CE6 NKWaitForMultipleObjects rejects fWaitAll == TRUE with
    // WAIT_FAILED/ERROR_INVALID_PARAMETER before it considers timeout/readiness.
    SetLastError(0);
    DWORD r = WaitForMultipleObjects(2, handles, TRUE, 30);
    if (r != WAIT_FAILED) return FixtureFail(7102);
    if (GetLastError() != ERROR_INVALID_PARAMETER) return FixtureFail(7104);

    SetEvent(e2);
    SetLastError(0);
    r = WaitForMultipleObjects(2, handles, TRUE, 1000);
    if (r != WAIT_FAILED) return FixtureFail(7103);
    if (GetLastError() != ERROR_INVALID_PARAMETER) return FixtureFail(7105);

    CloseHandle(e2);
    CloseHandle(e1);
    return FIXTURE_OK;
}
