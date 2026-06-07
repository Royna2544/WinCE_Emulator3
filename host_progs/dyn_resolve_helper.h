#ifndef HOST_PROGS_DYN_RESOLVE_HELPER_H
#define HOST_PROGS_DYN_RESOLVE_HELPER_H

#include <windows.h>
#include <tlhelp32.h>

typedef HANDLE (WINAPI *PFN_OpenProcess)(DWORD, BOOL, DWORD);
typedef BOOL (WINAPI *PFN_TerminateProcess)(HANDLE, UINT);
typedef HANDLE (WINAPI *PFN_CreateToolhelp32Snapshot)(DWORD, DWORD);
typedef BOOL (WINAPI *PFN_Process32First)(HANDLE, LPPROCESSENTRY32);
typedef BOOL (WINAPI *PFN_Process32Next)(HANDLE, LPPROCESSENTRY32);

FARPROC ResolveRuntimeProc(LPCWSTR dll_name, LPCWSTR proc_name);
HANDLE DynOpenProcess(DWORD access, BOOL inherit_handle, DWORD process_id);
BOOL DynTerminateProcess(HANDLE process, UINT exit_code);
HANDLE DynCreateToolhelp32Snapshot(DWORD flags, DWORD process_id);
BOOL DynProcess32First(HANDLE snapshot, LPPROCESSENTRY32 entry);
BOOL DynProcess32Next(HANDLE snapshot, LPPROCESSENTRY32 entry);

#endif
