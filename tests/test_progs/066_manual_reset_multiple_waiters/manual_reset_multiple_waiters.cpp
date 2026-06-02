#include <windows.h>
#include "../common/fixture_status.h"

struct Shared {
    HANDLE eventHandle;
    volatile LONG completed;
};

static DWORD WINAPI Waiter(LPVOID p) {
    Shared* s = (Shared*)p;
    DWORD r = WaitForSingleObject(s->eventHandle, 5000);
    if (r == WAIT_OBJECT_0) InterlockedIncrement(&s->completed);
    return r;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    Shared s;
    s.completed = 0;
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(6601);

    DWORD tid1 = 0, tid2 = 0;
    HANDLE t1 = CreateThread(0, 0, Waiter, &s, 0, &tid1);
    HANDLE t2 = CreateThread(0, 0, Waiter, &s, 0, &tid2);
    if (!t1 || !t2) return FixtureFail(6602);

    Sleep(20);
    SetEvent(s.eventHandle);

    WaitForSingleObject(t1, 5000);
    WaitForSingleObject(t2, 5000);

    if (s.completed != 2) return FixtureFail(6603);

    CloseHandle(t1);
    CloseHandle(t2);
    CloseHandle(s.eventHandle);
    return FIXTURE_OK;
}
