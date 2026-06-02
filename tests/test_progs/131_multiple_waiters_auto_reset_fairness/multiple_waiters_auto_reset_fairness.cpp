#include <windows.h>
#include "../common/fixture_status.h"
struct Shared { HANDLE e; volatile LONG count; };
static DWORD WINAPI Waiter(LPVOID p) {
    Shared* s = (Shared*)p;
    if (WaitForSingleObject(s->e, 1000) == WAIT_OBJECT_0) InterlockedIncrement((LONG*)&s->count);
    return 0;
}
int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    Shared s; s.count = 0; s.e = CreateEventW(0, FALSE, FALSE, 0);
    if (!s.e) return FixtureFail(13101);
    DWORD tid1=0, tid2=0;
    HANDLE t1 = CreateThread(0,0,Waiter,&s,0,&tid1);
    HANDLE t2 = CreateThread(0,0,Waiter,&s,0,&tid2);
    Sleep(20);
    SetEvent(s.e);
    Sleep(80);
    if (s.count != 1) return FixtureFail(13102);
    if (s.count < 2) SetEvent(s.e);
    WaitForSingleObject(t1, 5000);
    WaitForSingleObject(t2, 5000);
    if (s.count != 2) return FixtureFail(13103);
    CloseHandle(t1); CloseHandle(t2); CloseHandle(s.e);
    return FIXTURE_OK;
}
