#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    RECT a;
    RECT b;
    RECT c;
    RECT copy;

    if (!SetRect(&a, 10, 20, 70, 90)) {
        return FixtureFail(1);
    }
    if (a.left != 10 || a.top != 20 || a.right != 70 || a.bottom != 90) {
        return FixtureFail(2);
    }

    if (!CopyRect(&copy, &a) || !EqualRect(&copy, &a)) {
        return FixtureFail(3);
    }

    if (!OffsetRect(&copy, -5, 3)) {
        return FixtureFail(4);
    }
    if (copy.left != 5 || copy.top != 23 || copy.right != 65 || copy.bottom != 93) {
        return FixtureFail(5);
    }

    SetRect(&b, 40, 50, 120, 140);
    if (!IntersectRect(&c, &a, &b)) {
        return FixtureFail(6);
    }
    if (c.left != 40 || c.top != 50 || c.right != 70 || c.bottom != 90) {
        return FixtureFail(7);
    }

    SetRectEmpty(&c);
    if (!IsRectEmpty(&c)) {
        return FixtureFail(8);
    }

    SetRect(&b, 200, 200, 210, 210);
    if (IntersectRect(&c, &a, &b)) {
        return FixtureFail(9);
    }
    if (!IsRectEmpty(&c)) {
        return FixtureFail(10);
    }

    return FIXTURE_OK;
}
