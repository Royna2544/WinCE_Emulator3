#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE e1 = CreateEventW(0, TRUE, TRUE, 0);
    HANDLE e2 = CreateEventW(0, TRUE, FALSE, 0);
    if (!e1 || !e2) return FixtureFail(7101);

    HANDLE handles[2] = { e1, e2 };

    DWORD r = WaitForMultipleObjects(2, handles, TRUE, 30);
    if (r != WAIT_TIMEOUT) return FixtureFail(7102);

    SetEvent(e2);
    r = WaitForMultipleObjects(2, handles, TRUE, 1000);
    if (r != WAIT_OBJECT_0) return FixtureFail(7103);

    CloseHandle(e2);
    CloseHandle(e1);
    return FIXTURE_OK;
}
