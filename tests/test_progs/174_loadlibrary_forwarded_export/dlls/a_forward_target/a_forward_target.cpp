#include <windows.h>

extern "C" DWORD WINAPI TargetByName(DWORD value) {
    return value + 0x1200;
}

extern "C" DWORD WINAPI TargetByOrdinal(DWORD value) {
    return value + 0x3400;
}
