#include <afxwin.h>
#include "resource.h"

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
