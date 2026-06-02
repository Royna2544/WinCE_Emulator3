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
    r = WaitForMultipleObjects(2, handles, TRUE, 1000);
    if (r != WAIT_OBJECT_0) return FixtureFail(4303);

    HANDLE mutex = CreateMutexW(0, TRUE, L"Fixture043MutexW");
    if (!mutex) return FixtureFail(4304);

    if (!ReleaseMutex(mutex)) return FixtureFail(4305);

    CloseHandle(mutex);
    CloseHandle(e2);
    CloseHandle(e1);
    return FIXTURE_OK;
}
