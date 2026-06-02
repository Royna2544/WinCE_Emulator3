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

static int FileExistsW(const wchar_t* path) {
    DWORD attrs;
    if (!path) return 0;
    attrs = GetFileAttributesW(path);
    return attrs != 0xFFFFFFFF && ((attrs & FILE_ATTRIBUTE_DIRECTORY) == 0);
}

static int WriteMarkerFileW(const wchar_t* path, const char* bytes) {
    HANDLE file;
    DWORD length = 0;
    DWORD written = 0;
    if (!path || !bytes) return 0;
    while (bytes[length]) ++length;
    file = CreateFileW(path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (file == INVALID_HANDLE_VALUE) return 0;
    if (!WriteFile(file, bytes, length, &written, 0)) {
        CloseHandle(file);
        return 0;
    }
    CloseHandle(file);
    return written == length;
}

static int WideContains(const wchar_t* haystack, const wchar_t* needle) {
    const wchar_t* h;
    if (!haystack || !needle) return 0;
    if (!*needle) return 1;
    for (h = haystack; *h; ++h) {
        const wchar_t* a = h;
        const wchar_t* b = needle;
        while (*a && *b && *a == *b) {
            ++a;
            ++b;
        }
        if (!*b) return 1;
    }
    return 0;
}
