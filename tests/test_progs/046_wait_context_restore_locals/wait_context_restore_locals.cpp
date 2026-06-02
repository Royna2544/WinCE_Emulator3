#include <windows.h>
#include "../common/fixture_status.h"

struct Shared {
    HANDLE eventHandle;
    volatile DWORD marker;
};

static DWORD WINAPI WorkerProc(LPVOID p) {
    Shared* s = (Shared*)p;
    Sleep(20);
    s->marker = 0x44556677;
    SetEvent(s->eventHandle);
    return 0;
}

static DWORD PressureFunction(HANDLE eventHandle, volatile DWORD* marker) {
    volatile DWORD a = 0x11111111;
    volatile DWORD b = 0x22222222;
    volatile DWORD c = 0x33333333;
    volatile DWORD d = 0x44444444;
    volatile DWORD e = 0x55555555;
    volatile DWORD f = 0x66666666;
    DWORD sumBefore = a ^ b ^ c ^ d ^ e ^ f;

    DWORD wait = WaitForSingleObject(eventHandle, 5000);

    DWORD sumAfter = a ^ b ^ c ^ d ^ e ^ f;

    if (wait != WAIT_OBJECT_0) return FixtureFail(4601);
    if (*marker != 0x44556677) return FixtureFail(4602);
    if (a != 0x11111111 || b != 0x22222222 || c != 0x33333333) return FixtureFail(4603);
    if (d != 0x44444444 || e != 0x55555555 || f != 0x66666666) return FixtureFail(4604);
    if (sumBefore != sumAfter) return FixtureFail(4605);

    return FIXTURE_OK;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    Shared s;
    ZeroMemory(&s, sizeof(s));
    s.eventHandle = CreateEventW(0, TRUE, FALSE, 0);
    if (!s.eventHandle) return FixtureFail(4611);

    DWORD tid = 0;
    HANDLE th = CreateThread(0, 0, WorkerProc, &s, 0, &tid);
    if (!th) return FixtureFail(4612);

    DWORD rc = PressureFunction(s.eventHandle, &s.marker);
    WaitForSingleObject(th, 5000);

    CloseHandle(th);
    CloseHandle(s.eventHandle);
    return rc;
}
