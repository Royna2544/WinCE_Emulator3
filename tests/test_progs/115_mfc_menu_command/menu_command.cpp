#include <afxwin.h>
#include "../common/fixture_status.h"

#define ID_FIXTURE_COMMAND 11501

class CFixtureWnd : public CWnd {
public:
    DWORD seen;
    CFixtureWnd() : seen(0) {}

    afx_msg int OnCreate(LPCREATESTRUCT cs) {
        seen |= 1;
        return 0;
    }

    afx_msg LRESULT OnUser(WPARAM wp, LPARAM lp) {
        seen |= 2;
        return 0x07300 + (LRESULT)wp;
    }

    afx_msg void OnTimer(UINT_PTR id) {
        seen |= 4;
        KillTimer(1);
        PostMessage(WM_CLOSE);
    }

    afx_msg void OnCommandFixture() {
        seen |= 8;
    }

    DECLARE_MESSAGE_MAP()
};

BEGIN_MESSAGE_MAP(CFixtureWnd, CWnd)
    ON_WM_CREATE()
    ON_WM_TIMER()
    ON_COMMAND(ID_FIXTURE_COMMAND, OnCommandFixture)
    ON_MESSAGE(WM_USER + 115, OnUser)
END_MESSAGE_MAP()

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrev, LPWSTR cmd, int show) {
    if (!AfxWinInit(hInstance, hPrev, cmd, show)) return FixtureFail(11501);

    CString cls = AfxRegisterWndClass(0);
    CFixtureWnd wnd;
    if (!wnd.CreateEx(0, cls, _T("mfc fixture 115"), WS_VISIBLE, CRect(0,0,120,80), NULL, 0))
        return FixtureFail(11502);

    if ((wnd.seen & 1) == 0) return FixtureFail(11503);
    if (wnd.SendMessage(WM_USER + 115, 7, 0) != 0x07307) return FixtureFail(11504);
    if ((wnd.seen & 2) == 0) return FixtureFail(11505);

    wnd.SendMessage(WM_COMMAND, ID_FIXTURE_COMMAND, 0);
    if ((wnd.seen & 8) == 0) return FixtureFail(11506);

    wnd.SetTimer(1, 1, NULL);
    MSG msg;
    DWORD spins = 0;
    while ((wnd.seen & 4) == 0 && spins++ < 100) {
        while (PeekMessageW(&msg, 0, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(1);
    }
    if ((wnd.seen & 4) == 0) return FixtureFail(11507);

    if (wnd.m_hWnd) wnd.DestroyWindow();
    return FIXTURE_OK;
}
