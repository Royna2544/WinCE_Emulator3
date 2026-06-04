#include <windows.h>
#include "../common/fixture_status.h"

struct SleepArgs {
    volatile DWORD* state;
};

static DWORD WINAPI SleepWorker(LPVOID param) {
    SleepArgs* args = (SleepArgs*)param;
    *(args->state) = 1;
    Sleep(INFINITE);
    *(args->state) = 2;
    return 0x167;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    volatile DWORD state = 0;
    SleepArgs args;
    args.state = &state;

    DWORD threadId = 0;
    HANDLE thread = CreateThread(0, 0, SleepWorker, &args, 0, &threadId);
    if (!thread) return FixtureFail(16701);

    for (int i = 0; i < 100 && state != 1; ++i) {
        Sleep(1);
    }
    if (state != 1) return FixtureFail(16702);

    DWORD waitBeforeResume = WaitForSingleObject(thread, 0);
    if (waitBeforeResume != WAIT_TIMEOUT) return FixtureFail(16703);

    DWORD previousSuspend = ResumeThread(thread);
    if (previousSuspend != 1) return FixtureFail(16704);

    DWORD waitAfterResume = WaitForSingleObject(thread, 5000);
    if (waitAfterResume != WAIT_OBJECT_0) return FixtureFail(16705);
    if (state != 2) return FixtureFail(16706);

    DWORD exitCode = 0;
    if (!GetExitCodeThread(thread, &exitCode)) return FixtureFail(16707);
    CloseHandle(thread);
    return exitCode == 0x167 ? 0 : FixtureFail(16708);
}
