#include <windows.h>
#include "../common/fixture_status.h"

struct WaitArgs {
    HANDLE handle;
    volatile DWORD* state;
    DWORD value;
};

static DWORD WINAPI WaitAndStore(LPVOID param) {
    WaitArgs* args = (WaitArgs*)param;
    DWORD wait = WaitForSingleObject(args->handle, 5000);
    if (wait != WAIT_OBJECT_0) {
        *(args->state) = 0xff000000 | wait;
        return wait;
    }
    *(args->state) = args->value;
    if (args->value == 0x33333333) {
        ReleaseMutex(args->handle);
    }
    return args->value;
}

static int RunOne(HANDLE handle, volatile DWORD* state, DWORD value, DWORD failBase) {
    WaitArgs args;
    args.handle = handle;
    args.state = state;
    args.value = value;

    DWORD threadId = 0;
    HANDLE thread = CreateThread(0, 0, WaitAndStore, &args, 0, &threadId);
    if (!thread) {
        return FixtureFail(failBase + 1);
    }

    DWORD threadWait = WaitForSingleObject(thread, 5000);
    if (threadWait != WAIT_OBJECT_0) {
        CloseHandle(thread);
        return FixtureFail(failBase + 2);
    }
    if (*state != value) {
        CloseHandle(thread);
        return FixtureFail(failBase + 3);
    }

    CloseHandle(thread);
    return 0;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    volatile DWORD state = 0;

    HANDLE eventHandle = CreateEventW(0, FALSE, FALSE, L"Fixture164Event");
    if (!eventHandle) return FixtureFail(16401);

    WaitArgs eventArgs;
    eventArgs.handle = eventHandle;
    eventArgs.state = &state;
    eventArgs.value = 0x11111111;
    DWORD eventThreadId = 0;
    HANDLE eventThread = CreateThread(0, 0, WaitAndStore, &eventArgs, 0, &eventThreadId);
    if (!eventThread) return FixtureFail(16402);
    Sleep(1);
    if (!SetEvent(eventHandle)) return FixtureFail(16403);
    if (WaitForSingleObject(eventThread, 5000) != WAIT_OBJECT_0) return FixtureFail(16404);
    if (state != 0x11111111) return FixtureFail(16405);
    CloseHandle(eventThread);
    CloseHandle(eventHandle);

    state = 0;
    HANDLE semaphore = CreateSemaphoreW(0, 0, 1, L"Fixture164Semaphore");
    if (!semaphore) return FixtureFail(16410);
    WaitArgs semArgs;
    semArgs.handle = semaphore;
    semArgs.state = &state;
    semArgs.value = 0x22222222;
    DWORD semThreadId = 0;
    HANDLE semThread = CreateThread(0, 0, WaitAndStore, &semArgs, 0, &semThreadId);
    if (!semThread) return FixtureFail(16411);
    Sleep(1);
    if (!ReleaseSemaphore(semaphore, 1, 0)) return FixtureFail(16412);
    if (WaitForSingleObject(semThread, 5000) != WAIT_OBJECT_0) return FixtureFail(16413);
    if (state != 0x22222222) return FixtureFail(16414);
    CloseHandle(semThread);
    CloseHandle(semaphore);

    state = 0;
    HANDLE mutex = CreateMutexW(0, TRUE, L"Fixture164Mutex");
    if (!mutex) return FixtureFail(16420);
    WaitArgs mutexArgs;
    mutexArgs.handle = mutex;
    mutexArgs.state = &state;
    mutexArgs.value = 0x33333333;
    DWORD mutexThreadId = 0;
    HANDLE mutexThread = CreateThread(0, 0, WaitAndStore, &mutexArgs, 0, &mutexThreadId);
    if (!mutexThread) return FixtureFail(16421);
    Sleep(1);
    if (!ReleaseMutex(mutex)) return FixtureFail(16422);
    if (WaitForSingleObject(mutexThread, 5000) != WAIT_OBJECT_0) return FixtureFail(16423);
    if (state != 0x33333333) return FixtureFail(16424);
    CloseHandle(mutexThread);
    CloseHandle(mutex);

    return FIXTURE_OK;
}
