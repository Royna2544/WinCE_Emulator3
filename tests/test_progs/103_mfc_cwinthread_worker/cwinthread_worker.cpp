#include <afxwin.h>
#include "../common/fixture_status.h"
static volatile DWORD g_workerValue = 0;
UINT WorkerProc(LPVOID p) {
    DWORD slot = (DWORD)p;
    TlsSetValue(slot, (LPVOID)0x103103);
    g_workerValue = (DWORD)TlsGetValue(slot);
    return 0;
}
int WINAPI WinMain(HINSTANCE h, HINSTANCE p, LPWSTR c, int s) {
    if (!AfxWinInit(h, p, c, s)) return FixtureFail(10301);
    DWORD slot = TlsAlloc();
    if (slot == TLS_OUT_OF_INDEXES) return FixtureFail(10302);
    TlsSetValue(slot, (LPVOID)0xaaaa);
    CWinThread* t = AfxBeginThread(WorkerProc, (LPVOID)slot);
    if (!t) return FixtureFail(10303);
    WaitForSingleObject(t->m_hThread, 5000);
    if ((DWORD)TlsGetValue(slot) != 0xaaaa) return FixtureFail(10304);
    if (g_workerValue != 0x103103) return FixtureFail(10305);
    TlsFree(slot);
    return FIXTURE_OK;
}
