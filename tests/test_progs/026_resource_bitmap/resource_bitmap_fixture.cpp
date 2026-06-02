#include <windows.h>
#include "resource.h"
#include "../common/fixture_status.h"

#ifndef IMAGE_BITMAP
#define IMAGE_BITMAP 0
#endif
#ifndef LR_CREATEDIBSECTION
#define LR_CREATEDIBSECTION 0x00002000
#endif

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int) {
    wchar_t text[128];
    ZeroMemory(text, sizeof(text));

    int len = LoadStringW(hInstance, IDS_RESOURCE_BITMAP_OK, text, 128);
    if (len <= 0) return FixtureFail(2601);
    if (!WideEqAscii(text, "resource-bitmap-ok")) return FixtureFail(2602);

    HRSRC dataRes = FindResourceW(hInstance, MAKEINTRESOURCEW(IDR_TEST_DATA), MAKEINTRESOURCEW(10));
    if (!dataRes) return FixtureFail(2603);
    DWORD dataSize = SizeofResource(hInstance, dataRes);
    if (dataSize < 6) return FixtureFail(2604);
    HGLOBAL dataHandle = LoadResource(hInstance, dataRes);
    if (!dataHandle) return FixtureFail(2605);

    HBITMAP bmp = (HBITMAP)LoadImageW(hInstance, MAKEINTRESOURCEW(IDB_TEST_4X4), IMAGE_BITMAP, 0, 0, LR_CREATEDIBSECTION);
    if (!bmp) return FixtureFail(2606);

    BITMAP bm;
    ZeroMemory(&bm, sizeof(bm));
    if (!GetObjectW(bmp, sizeof(bm), &bm)) return FixtureFail(2607);
    if (bm.bmWidth != 4 || bm.bmHeight != 4) return FixtureFail(2608);

    DeleteObject(bmp);
    return FIXTURE_OK;
}
