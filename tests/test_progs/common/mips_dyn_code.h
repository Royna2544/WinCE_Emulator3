#pragma once

#include <windows.h>
#include <string.h>

#ifndef PAGE_EXECUTE_READWRITE
#define PAGE_EXECUTE_READWRITE PAGE_READWRITE
#endif

static void StoreLe32(unsigned char* out, DWORD word) {
    out[0] = (unsigned char)(word & 0xff);
    out[1] = (unsigned char)((word >> 8) & 0xff);
    out[2] = (unsigned char)((word >> 16) & 0xff);
    out[3] = (unsigned char)((word >> 24) & 0xff);
}

static void WriteMipsWordsLe(unsigned char* out, const DWORD* words, DWORD count) {
    for (DWORD i = 0; i < count; ++i) {
        StoreLe32(out + i * 4, words[i]);
    }
}

typedef int (*MipsNoArgFunction)();

static int RunMipsWords(const DWORD* words, DWORD wordCount, DWORD* outAllocError) {
    const DWORD byteCount = wordCount * 4;
    if (outAllocError) {
        *outAllocError = 0;
    }

    void* code = VirtualAlloc(0, byteCount, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!code) {
        code = VirtualAlloc(0, byteCount, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    }
    if (!code) {
        if (outAllocError) {
            *outAllocError = GetLastError();
        }
        return -1;
    }

    WriteMipsWordsLe((unsigned char*)code, words, wordCount);

    /*
       On real MIPS, self-modifying/generated code generally needs an I-cache flush.
       Windows CE commonly exposes FlushInstructionCache through coredll.
       If an SDK lacks the declaration, add it in the local project headers.
    */
    FlushInstructionCache(GetCurrentProcess(), code, byteCount);

    MipsNoArgFunction fn = (MipsNoArgFunction)code;
    int result = fn();

    VirtualFree(code, 0, MEM_RELEASE);
    return result;
}
