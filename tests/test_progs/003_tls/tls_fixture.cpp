#include <windows.h>
#include "../common/fixture_status.h"

struct WorkerArgs {
    DWORD slot;
    HANDLE ready;
    DWORD observedMainValue;
    DWORD workerValue;
};

static DWORD WINAPI WorkerThread(LPVOID param) {
    WorkerArgs* args = (WorkerArgs*)param;

    args->observedMainValue = (DWORD)TlsGetValue(args->slot);

    TlsSetValue(args->slot, (LPVOID)0x22222222);
    args->workerValue = (DWORD)TlsGetValue(args->slot);

    SetEvent(args->ready);
    return 0;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    DWORD slot = TlsAlloc();
    if (slot == TLS_OUT_OF_INDEXES) {
        return FixtureFail(1);
    }

    TlsSetValue(slot, (LPVOID)0x11111111);
    if ((DWORD)TlsGetValue(slot) != 0x11111111) {
        return FixtureFail(2);
    }

    HANDLE ready = CreateEventW(0, TRUE, FALSE, 0);
    if (!ready) {
        return FixtureFail(3);
    }

    WorkerArgs args;
    args.slot = slot;
    args.ready = ready;
    args.observedMainValue = 0xcccccccc;
    args.workerValue = 0xcccccccc;

    DWORD threadId = 0;
    HANDLE thread = CreateThread(0, 0, WorkerThread, &args, 0, &threadId);
    if (!thread) {
        return FixtureFail(4);
    }

    DWORD wait = WaitForSingleObject(ready, 5000);
    if (wait != WAIT_OBJECT_0) {
        return FixtureFail(5);
    }

    /*
       Main thread's TLS value must remain isolated from worker's TLS value.
       A broken global slot map often returns 0x22222222 here.
    */
    if ((DWORD)TlsGetValue(slot) != 0x11111111) {
        return FixtureFail(6);
    }

    /*
       A new worker thread should not see main's TLS slot value before it sets its own.
       Depending on CE behavior, unset normally reads as 0.
    */
    if (args.observedMainValue != 0) {
        return FixtureFail(7);
    }

    if (args.workerValue != 0x22222222) {
        return FixtureFail(8);
    }

    WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    CloseHandle(ready);
    TlsFree(slot);

    return FIXTURE_OK;
}
