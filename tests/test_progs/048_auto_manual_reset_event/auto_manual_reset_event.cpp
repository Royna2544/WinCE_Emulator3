#include <windows.h>
#include "../common/fixture_status.h"

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    HANDLE autoEvent = CreateEventW(0, FALSE, FALSE, 0);
    if (!autoEvent) return FixtureFail(4801);

    SetEvent(autoEvent);
    if (WaitForSingleObject(autoEvent, 1000) != WAIT_OBJECT_0) return FixtureFail(4802);
    if (WaitForSingleObject(autoEvent, 20) != WAIT_TIMEOUT) return FixtureFail(4803);

    HANDLE manualEvent = CreateEventW(0, TRUE, FALSE, 0);
    if (!manualEvent) return FixtureFail(4804);

    SetEvent(manualEvent);
    if (WaitForSingleObject(manualEvent, 1000) != WAIT_OBJECT_0) return FixtureFail(4805);
    if (WaitForSingleObject(manualEvent, 1000) != WAIT_OBJECT_0) return FixtureFail(4806);

    ResetEvent(manualEvent);
    if (WaitForSingleObject(manualEvent, 20) != WAIT_TIMEOUT) return FixtureFail(4807);

    CloseHandle(manualEvent);
    CloseHandle(autoEvent);
    return FIXTURE_OK;
}
