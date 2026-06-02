#pragma once

#include <windows.h>

#define FIXTURE_OK 0
#define FIXTURE_FAIL_BASE 0x1000

#ifndef TLS_OUT_OF_INDEXES
#define TLS_OUT_OF_INDEXES ((DWORD)0xFFFFFFFF)
#endif

static DWORD FixtureFail(DWORD code) {
    return FIXTURE_FAIL_BASE + code;
}

static void FixtureTouch(volatile DWORD* sink, DWORD value) {
    if (sink) {
        *sink = value;
    }
}

static int WideEqAscii(const wchar_t* wide, const char* ascii) {
    if (!wide || !ascii) return 0;
    while (*wide && *ascii) {
        if ((char)*wide != *ascii) return 0;
        ++wide;
        ++ascii;
    }
    return *wide == 0 && *ascii == 0;
}

static int BytesEq(const BYTE* left, const BYTE* right, DWORD count) {
    DWORD i;
    if (!left || !right) return 0;
    for (i = 0; i < count; ++i) {
        if (left[i] != right[i]) return 0;
    }
    return 1;
}
