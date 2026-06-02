#include <windows.h>
#include "../common/fixture_status.h"
static volatile LONG g_count = 0;
static DWORD WINAPI Worker(LPVOID p) { InterlockedIncrement((LONG*)&g_count); return 0; }
int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    int i;
    for (i = 0; i < 80; ++i) {
        DWORD slot = TlsAlloc();
        if (slot == TLS_OUT_OF_INDEXES) return FixtureFail(15701);
        TlsSetValue(slot, (LPVOID)(0x09d000 + i));
        if ((DWORD)TlsGetValue(slot) != (DWORD)(0x09d000 + i)) return FixtureFail(15702);
        TlsFree(slot);
        DWORD tid = 0;
        HANDLE th = CreateThread(0, 0, Worker, 0, 0, &tid);
        if (!th) return FixtureFail(15703);
        WaitForSingleObject(th, 5000);
        CloseHandle(th);
    }
    if (g_count != 80) return FixtureFail(15704);
    return FIXTURE_OK;
}
