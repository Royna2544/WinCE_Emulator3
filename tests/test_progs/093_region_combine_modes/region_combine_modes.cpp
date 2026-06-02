#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HRGN a = CreateRectRgn(0, 0, 50, 50);
    HRGN b = CreateRectRgn(25, 25, 75, 75);
    HRGN out = CreateRectRgn(0, 0, 0, 0);
    if (!a || !b || !out) return FixtureFail(9301);

    if (CombineRgn(out, a, b, RGN_AND) == ERROR) return FixtureFail(9302);
    if (CombineRgn(out, a, b, RGN_OR) == ERROR) return FixtureFail(9303);
    if (CombineRgn(out, a, b, RGN_XOR) == ERROR) return FixtureFail(9304);
    if (CombineRgn(out, a, b, RGN_DIFF) == ERROR) return FixtureFail(9305);
    if (CombineRgn(out, a, b, RGN_COPY) == ERROR) return FixtureFail(9306);

    RECT rc;
    if (!GetRgnBox(out, &rc)) return FixtureFail(9307);

    DeleteObject(out);
    DeleteObject(b);
    DeleteObject(a);
    return FIXTURE_OK;
}
