#include <windows.h>
#include "../common/fixture_status.h"

#ifndef ERROR_NOT_OWNER
#define ERROR_NOT_OWNER 288L
#endif

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE mutex = CreateMutexW(0, TRUE, L"FixtureMutex163");
    if (!mutex) return FixtureFail(16301);
    if (GetLastError() != 0) return FixtureFail(16302);

    if (WaitForSingleObject(mutex, 0) != WAIT_OBJECT_0) return FixtureFail(16303);
    if (WaitForSingleObject(mutex, 0) != WAIT_OBJECT_0) return FixtureFail(16304);

    if (!ReleaseMutex(mutex)) return FixtureFail(16305);
    if (!ReleaseMutex(mutex)) return FixtureFail(16306);
    if (!ReleaseMutex(mutex)) return FixtureFail(16307);

    SetLastError(0);
    if (ReleaseMutex(mutex)) return FixtureFail(16308);
    if (GetLastError() != ERROR_NOT_OWNER) return FixtureFail(16309);

    if (WaitForSingleObject(mutex, 0) != WAIT_OBJECT_0) return FixtureFail(16310);
    if (!ReleaseMutex(mutex)) return FixtureFail(16311);

    CloseHandle(mutex);
    return FIXTURE_OK;
}
