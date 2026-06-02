#include <windows.h>
#include "../common/fixture_status.h"

static DWORD WINAPI Worker(LPVOID) {
    Sleep(10);
    return 0;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    DWORD tid = 0;
    HANDLE th = CreateThread(0, 0, Worker, 0, 0, &tid);
    if (!th) return FixtureFail(6901);

    int oldPriority = GetThreadPriority(th);
    if (oldPriority == THREAD_PRIORITY_ERROR_RETURN) return FixtureFail(6902);

    SetThreadPriority(th, THREAD_PRIORITY_ABOVE_NORMAL);
    int newPriority = GetThreadPriority(th);
    if (newPriority == THREAD_PRIORITY_ERROR_RETURN) return FixtureFail(6903);

    SetThreadPriority(th, oldPriority);
    WaitForSingleObject(th, 5000);
    CloseHandle(th);
    return FIXTURE_OK;
}
