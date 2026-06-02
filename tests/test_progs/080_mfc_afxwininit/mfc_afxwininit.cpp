#include <afxwin.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrev, LPWSTR cmd, int show) {
    if (!AfxWinInit(hInstance, hPrev, cmd, show)) {
        return FixtureFail(8001);
    }

    CString text = _T("mfc-ok");
    if (text.GetLength() != 6) {
        return FixtureFail(8002);
    }

    return FIXTURE_OK;
}
