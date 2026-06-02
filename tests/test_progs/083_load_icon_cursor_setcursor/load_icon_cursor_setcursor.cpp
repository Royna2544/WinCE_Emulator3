#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HCURSOR cursor = LoadCursorW(0, MAKEINTRESOURCEW(32512));
    if (!cursor) return FixtureFail(8301);

    HCURSOR oldCursor = SetCursor(cursor);
    (void)oldCursor;

    HICON icon = LoadIconW(0, MAKEINTRESOURCEW(32512));
    /*
       Some CE images may not have a stock application icon. The path is still useful.
    */
    (void)icon;

    return FIXTURE_OK;
}
