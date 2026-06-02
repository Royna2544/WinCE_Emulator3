#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    DWORD t0 = GetTickCount();
    Sleep(1);
    DWORD t1 = GetTickCount();

    LARGE_INTEGER freq;
    LARGE_INTEGER c0;
    LARGE_INTEGER c1;

    if (!QueryPerformanceFrequency(&freq)) {
        return FixtureFail(1);
    }
    if (!QueryPerformanceCounter(&c0)) {
        return FixtureFail(2);
    }
    if (!QueryPerformanceCounter(&c1)) {
        return FixtureFail(3);
    }
    if (freq.QuadPart == 0) {
        return FixtureFail(4);
    }

    /*
       Do not require t1 > t0. Some emulators may virtualize time coarsely.
       Only require that calls are valid and monotonic-ish.
    */
    if (t1 < t0) {
        return FixtureFail(5);
    }
    if (c1.QuadPart < c0.QuadPart) {
        return FixtureFail(6);
    }

    return FIXTURE_OK;
}
