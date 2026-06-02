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
