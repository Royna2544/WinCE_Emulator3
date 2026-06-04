#include <windows.h>
#include "../common/fixture_status.h"

static DWORD WINAPI Worker(LPVOID param) {
    volatile DWORD* marker = (volatile DWORD*)param;
    *marker = 0x165165;
    return 0x5a;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    volatile DWORD marker = 0;
    DWORD tid = 0;
    HANDLE thread = CreateThread(0, 0, Worker, (LPVOID)&marker, 0, &tid);
    if (!thread) return FixtureFail(16501);

    DWORD wait = WaitForSingleObject(thread, 5000);
    if (wait != WAIT_OBJECT_0) return FixtureFail(16502);
    if (marker != 0x165165) return FixtureFail(16503);

    DWORD exitCode = 0;
    if (!GetExitCodeThread(thread, &exitCode)) return FixtureFail(16504);
    if (exitCode != 0x5a) return FixtureFail(16505);

    HANDLE eventHandle = CreateEventW(0, FALSE, FALSE, 0);
    if (!eventHandle) return FixtureFail(16506);

    HANDLE waits[2];
    waits[0] = eventHandle;
    waits[1] = thread;
    wait = WaitForMultipleObjects(2, waits, FALSE, 0);
    if (wait != WAIT_OBJECT_0 + 1) return FixtureFail(16507);

    CloseHandle(eventHandle);
    CloseHandle(thread);
    return FIXTURE_OK;
}
