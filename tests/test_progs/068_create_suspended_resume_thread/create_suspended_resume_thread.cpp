#include <windows.h>
#include "../common/fixture_status.h"

struct Shared {
    volatile DWORD ran;
};

static DWORD WINAPI Worker(LPVOID p) {
    Shared* s = (Shared*)p;
    s->ran = 0x6868;
    return 0;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    Shared s;
    s.ran = 0;

#ifndef CREATE_SUSPENDED
#define CREATE_SUSPENDED 0x00000004
#endif

    DWORD tid = 0;
    HANDLE th = CreateThread(0, 0, Worker, &s, CREATE_SUSPENDED, &tid);
    if (!th) return FixtureFail(6801);

    Sleep(50);
    if (s.ran != 0) return FixtureFail(6802);

    if (ResumeThread(th) == 0xffffffff) return FixtureFail(6803);
    if (WaitForSingleObject(th, 5000) != WAIT_OBJECT_0) return FixtureFail(6804);
    if (s.ran != 0x6868) return FixtureFail(6805);

    CloseHandle(th);
    return FIXTURE_OK;
}
