#include <afxwin.h>
#include "../common/fixture_status.h"

class CFixtureFrame : public CFrameWnd {
public:
    CFixtureFrame() {
        Create(NULL, _T("MFC Fixture Window"), WS_OVERLAPPEDWINDOW, CRect(0, 0, 120, 80));
    }

    afx_msg int OnCreate(LPCREATESTRUCT lpCreateStruct) {
        if (CFrameWnd::OnCreate(lpCreateStruct) == -1) return -1;
        PostMessage(WM_CLOSE);
        return 0;
    }

    DECLARE_MESSAGE_MAP()
};

BEGIN_MESSAGE_MAP(CFixtureFrame, CFrameWnd)
    ON_WM_CREATE()
END_MESSAGE_MAP()

class CFixtureApp : public CWinApp {
public:
    BOOL InitInstance() {
        CFixtureFrame* frame = new CFixtureFrame();
        m_pMainWnd = frame;
        frame->ShowWindow(SW_SHOW);
        frame->UpdateWindow();
        return TRUE;
    }
};

CFixtureApp theApp;

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrev, LPWSTR cmd, int show) {
    if (!AfxWinInit(hInstance, hPrev, cmd, show)) {
        return FixtureFail(8101);
    }
    if (!theApp.InitInstance()) {
        return FixtureFail(8102);
    }
    return FIXTURE_OK;
}
