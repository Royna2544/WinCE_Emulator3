#include "dyn_resolve_helper.h"

#pragma comment(lib, "coredll.lib")

static FARPROC ResolveFromDll(LPCWSTR dll_name, LPCWSTR proc_name) {
    HMODULE module = LoadLibraryW(dll_name);
    if (!module) {
        return 0;
    }
    return GetProcAddress(module, proc_name);
}

FARPROC ResolveRuntimeProc(LPCWSTR dll_name, LPCWSTR proc_name) {
    FARPROC proc = ResolveFromDll(dll_name, proc_name);
    if (proc) {
        return proc;
    }
    return ResolveFromDll(L"coredll.dll", proc_name);
}

HANDLE DynOpenProcess(DWORD access, BOOL inherit_handle, DWORD process_id) {
    PFN_OpenProcess proc = (PFN_OpenProcess)ResolveRuntimeProc(L"coredll.dll", L"OpenProcess");
    if (!proc) {
        SetLastError(ERROR_PROC_NOT_FOUND);
        return 0;
    }
    return proc(access, inherit_handle, process_id);
}

BOOL DynTerminateProcess(HANDLE process, UINT exit_code) {
    PFN_TerminateProcess proc = (PFN_TerminateProcess)ResolveRuntimeProc(L"coredll.dll", L"TerminateProcess");
    if (!proc) {
        SetLastError(ERROR_PROC_NOT_FOUND);
        return FALSE;
    }
    return proc(process, exit_code);
}

HANDLE DynCreateToolhelp32Snapshot(DWORD flags, DWORD process_id) {
    PFN_CreateToolhelp32Snapshot proc =
        (PFN_CreateToolhelp32Snapshot)ResolveRuntimeProc(L"toolhelp.dll", L"CreateToolhelp32Snapshot");
    if (!proc) {
        SetLastError(ERROR_PROC_NOT_FOUND);
        return INVALID_HANDLE_VALUE;
    }
    return proc(flags, process_id);
}

BOOL DynProcess32First(HANDLE snapshot, LPPROCESSENTRY32 entry) {
    PFN_Process32First proc = (PFN_Process32First)ResolveRuntimeProc(L"toolhelp.dll", L"Process32First");
    if (!proc) {
        SetLastError(ERROR_PROC_NOT_FOUND);
        return FALSE;
    }
    return proc(snapshot, entry);
}

BOOL DynProcess32Next(HANDLE snapshot, LPPROCESSENTRY32 entry) {
    PFN_Process32Next proc = (PFN_Process32Next)ResolveRuntimeProc(L"toolhelp.dll", L"Process32Next");
    if (!proc) {
        SetLastError(ERROR_PROC_NOT_FOUND);
        return FALSE;
    }
    return proc(snapshot, entry);
}
