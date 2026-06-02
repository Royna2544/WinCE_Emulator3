#include <windows.h>
#include "../common/fixture_status.h"

struct WorkerArgs {
    HANDLE eventHandle;
    volatile DWORD* value;
};

static DWORD WINAPI WorkerThread(LPVOID param) {
    WorkerArgs* args = (WorkerArgs*)param;
    *(args->value) = 0xabcdef01;
    SetEvent(args->eventHandle);
    return 0x55;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!eventHandle) {
        return FixtureFail(1);
    }

    volatile DWORD value = 0;
    WorkerArgs args;
    args.eventHandle = eventHandle;
    args.value = &value;

    DWORD threadId = 0;
    HANDLE thread = CreateThread(0, 0, WorkerThread, &args, 0, &threadId);
    if (!thread) {
        CloseHandle(eventHandle);
        return FixtureFail(2);
    }

    DWORD wait = WaitForSingleObject(eventHandle, 5000);
    if (wait != WAIT_OBJECT_0) {
        CloseHandle(thread);
        CloseHandle(eventHandle);
        return FixtureFail(3);
    }

    if (value != 0xabcdef01) {
        CloseHandle(thread);
        CloseHandle(eventHandle);
        return FixtureFail(4);
    }

    DWORD threadWait = WaitForSingleObject(thread, 5000);
    if (threadWait != WAIT_OBJECT_0) {
        CloseHandle(thread);
        CloseHandle(eventHandle);
        return FixtureFail(5);
    }

    CloseHandle(thread);
    CloseHandle(eventHandle);
    return FIXTURE_OK;
}
