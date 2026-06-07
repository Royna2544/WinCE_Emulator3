#include <windows.h>

extern "C" __declspec(dllimport) DWORD WINAPI ForwardByName(DWORD value);

extern "C" DWORD WINAPI UserCallsForward(DWORD value) {
    return ForwardByName(value) + 0x55;
}
