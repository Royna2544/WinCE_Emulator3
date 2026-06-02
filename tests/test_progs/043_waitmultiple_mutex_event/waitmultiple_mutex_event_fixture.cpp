#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE e1 = CreateEventW(0, TRUE, FALSE, 0);
    HANDLE e2 = CreateEventW(0, TRUE, TRUE, 0);
    if (!e1 || !e2) return FixtureFail(4301);

    HANDLE handles[2];
    handles[0] = e1;
    handles[1] = e2;

    DWORD r = WaitForMultipleObjects(2, handles, FALSE, 1000);
    if (r != WAIT_OBJECT_0 + 1) return FixtureFail(4302);

    SetEvent(e1);
    // CE6 NKWaitForMultipleObjects rejects fWaitAll == TRUE with
    // WAIT_FAILED/ERROR_INVALID_PARAMETER instead of waiting for all handles.
    SetLastError(0);
    r = WaitForMultipleObjects(2, handles, TRUE, 1000);
    if (r != WAIT_FAILED) return FixtureFail(4303);
    if (GetLastError() != ERROR_INVALID_PARAMETER) return FixtureFail(4306);

    HANDLE mutex = CreateMutexW(0, TRUE, L"Fixture043MutexW");
    if (!mutex) return FixtureFail(4304);

    if (!ReleaseMutex(mutex)) return FixtureFail(4305);

    CloseHandle(mutex);
    CloseHandle(e2);
    CloseHandle(e1);
    return FIXTURE_OK;
}
