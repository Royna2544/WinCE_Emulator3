#include <afxwin.h>
#include "resource.h"
#include "../common/fixture_status.h"

class CFixtureDialog : public CDialog {
public:
    CFixtureDialog() : CDialog(IDD_MFC_DIALOG) {}

    BOOL OnInitDialog() {
        CDialog::OnInitDialog();
        PostMessage(WM_COMMAND, IDOK, 0);
        return TRUE;
    }
};

class CFixtureApp : public CWinApp {
public:
    BOOL InitInstance() {
        CFixtureDialog dlg;
        dlg.DoModal();
        return FALSE;
    }
};

CFixtureApp theApp;

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrev, LPWSTR cmd, int show) {
    if (!AfxWinInit(hInstance, hPrev, cmd, show)) {
        return FixtureFail(8201);
    }
    theApp.InitInstance();
    return FIXTURE_OK;
}
