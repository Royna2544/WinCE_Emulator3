#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE h, HINSTANCE, LPWSTR, int) {
    HMENU menu = LoadMenuW(h, MAKEINTRESOURCEW(IDM_RADIO_MENU));
    if (!menu) return FixtureFail(8501);

    if (CheckMenuRadioItem(menu, IDM_RADIO_A, IDM_RADIO_C, IDM_RADIO_B, MF_BYCOMMAND) == 0xffffffff) {
        return FixtureFail(8502);
    }

    CheckMenuItem(menu, IDM_RADIO_C, MF_CHECKED);
    CheckMenuItem(menu, IDM_RADIO_C, MF_UNCHECKED);
    return FIXTURE_OK;
}
