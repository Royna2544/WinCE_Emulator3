#include <afxwin.h>
#include "../common/fixture_status.h"
int WINAPI WinMain(HINSTANCE h, HINSTANCE p, LPWSTR c, int s) {
    if (!AfxWinInit(h, p, c, s)) return FixtureFail(10101);
    DWORD slot = TlsAlloc();
    if (slot == TLS_OUT_OF_INDEXES) return FixtureFail(10102);
    TlsSetValue(slot, (LPVOID)0x10101010);
    if ((DWORD)TlsGetValue(slot) != 0x10101010) return FixtureFail(10103);
    CString text = _T("mfc-ok");
    if (text.GetLength() != 6) return FixtureFail(10104);
    TlsFree(slot);
    return FIXTURE_OK;
}
